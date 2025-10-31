// WASM Backend for Tong
// Compiles Tong AST to WebAssembly

use anyhow::{bail, Result};
use std::collections::HashMap;
use wasm_encoder::*;

use crate::parser::{Expr, Stmt, BinOp, Program};

/// WASM compiler context
pub struct WasmCompiler {
    /// Function signatures (name -> (param_count, has_return))
    functions: HashMap<String, (u32, bool)>,
    /// Local variables in current function scope (name -> local_index)
    locals: HashMap<String, u32>,
    /// Global variables (name -> global_index)
    globals: HashMap<String, u32>,
    /// Next available local index
    next_local: u32,
    /// Next available global index
    next_global: u32,
    /// String constants pool
    string_pool: Vec<String>,
    /// Memory offset for data section
    memory_offset: u32,
}

impl WasmCompiler {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            locals: HashMap::new(),
            globals: HashMap::new(),
            next_local: 0,
            next_global: 0,
            string_pool: Vec::new(),
            memory_offset: 0,
        }
    }

    /// Compile a Tong program to WASM binary
    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<u8>> {
        let mut module = Module::new();

        // Add type section
        let mut types = TypeSection::new();
        
        // Type 0: () -> ()
        types.ty().function(vec![], vec![]);
        
        // Type 1: (i32) -> ()  for print_i32
        types.ty().function(vec![ValType::I32], vec![]);
        
        // Type 2: (i64) -> ()  for print_i64
        types.ty().function(vec![ValType::I64], vec![]);
        
        // Type 3: (f64) -> ()  for print_f64
        types.ty().function(vec![ValType::F64], vec![]);
        
        // Type 4: (i32, i32) -> (i32)  for binary ops
        types.ty().function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
        
        module.section(&types);

        // Add import section for external functions (console.log, etc.)
        let mut imports = ImportSection::new();
        imports.import(
            "env",
            "print_i32",
            EntityType::Function(1),
        );
        imports.import(
            "env",
            "print_i64",
            EntityType::Function(2),
        );
        imports.import(
            "env",
            "print_f64",
            EntityType::Function(3),
        );
        module.section(&imports);

        // Add function section
        let mut functions = FunctionSection::new();
        
        // Add memory section
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: 1,
            maximum: Some(10),
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memories);

        // Scan for functions first
        self.scan_functions(&program.stmts)?;

        // Add function type indices
        for stmt in &program.stmts {
            if let Stmt::FnDef(_, _, _) = stmt {
                functions.function(0); // All functions return () for now
            }
        }
        
        // Add main function
        functions.function(0);
        module.section(&functions);

        // Add export section
        let mut exports = ExportSection::new();
        let main_fn_idx = self.functions.len() as u32 + 3; // After imports
        exports.export("main", ExportKind::Func, main_fn_idx);
        module.section(&exports);

        // Add code section
        let mut codes = CodeSection::new();
        
        // Compile user-defined functions
        for stmt in &program.stmts {
            if let Stmt::FnDef(name, params, body) = stmt {
                let func_code = self.compile_function(name, params, body)?;
                codes.function(&func_code);
            }
        }

        // Compile main function
        let main_code = self.compile_main(&program.stmts)?;
        codes.function(&main_code);

        module.section(&codes);

        // Add data section for string constants
        if !self.string_pool.is_empty() {
            let mut data = DataSection::new();
            let mut offset = 0;
            for s in &self.string_pool {
                data.active(
                    0,
                    &ConstExpr::i32_const(offset as i32),
                    s.as_bytes().to_vec(),
                );
                offset += s.len() as u32 + 1; // +1 for null terminator
            }
            module.section(&data);
        }

        Ok(module.finish())
    }

    /// Scan for function definitions to build function table
    fn scan_functions(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::FnDef(name, params, _) => {
                    self.functions.insert(
                        name.clone(),
                        (params.len() as u32, false),
                    );
                }
                Stmt::FnDefTyped(name, params, ret_type, _) => {
                    self.functions.insert(
                        name.clone(),
                        (params.len() as u32, ret_type.is_some()),
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Compile a function definition
    fn compile_function(&mut self, _name: &str, params: &[String], body: &[Stmt]) -> Result<Function> {
        // Reset local context
        self.locals.clear();
        self.next_local = 0;

        // Add parameters as locals
        for param in params {
            self.locals.insert(param.clone(), self.next_local);
            self.next_local += 1;
        }

        let mut func = Function::new(vec![]);
        
        // Compile function body
        for stmt in body {
            self.compile_stmt(stmt, &mut func)?;
        }

        // Ensure function returns
        func.instruction(&Instruction::End);

        Ok(func)
    }

    /// Compile the main program
    fn compile_main(&mut self, stmts: &[Stmt]) -> Result<Function> {
        self.locals.clear();
        self.next_local = 0;

        let mut func = Function::new(vec![]);

        for stmt in stmts {
            // Skip function definitions (already compiled)
            match stmt {
                Stmt::FnDef(_, _, _) | Stmt::FnDefTyped(_, _, _, _) => continue,
                _ => {}
            }
            self.compile_stmt(stmt, &mut func)?;
        }

        func.instruction(&Instruction::End);
        Ok(func)
    }

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt, func: &mut Function) -> Result<()> {
        match stmt {
            Stmt::Let(name, expr) => {
                // Allocate local variable
                let local_idx = self.next_local;
                self.locals.insert(name.clone(), local_idx);
                self.next_local += 1;

                // Compile expression
                self.compile_expr(expr, func)?;
                
                // Store to local
                func.instruction(&Instruction::LocalSet(local_idx));
            }

            Stmt::Var(name, expr) => {
                // Same as Let for now (mutable by default in WASM locals)
                let local_idx = self.next_local;
                self.locals.insert(name.clone(), local_idx);
                self.next_local += 1;

                self.compile_expr(expr, func)?;
                func.instruction(&Instruction::LocalSet(local_idx));
            }

            Stmt::Assign(name, expr) => {
                self.compile_expr(expr, func)?;
                
                if let Some(&local_idx) = self.locals.get(name) {
                    func.instruction(&Instruction::LocalSet(local_idx));
                } else if let Some(&global_idx) = self.globals.get(name) {
                    func.instruction(&Instruction::GlobalSet(global_idx));
                } else {
                    bail!("Undefined variable: {}", name);
                }
            }

            Stmt::Print(exprs) => {
                // Print each expression
                for expr in exprs {
                    self.compile_expr(expr, func)?;
                    
                    // Call appropriate print function based on type
                    // For now, assume i64 (we'd need type inference for better handling)
                    func.instruction(&Instruction::Call(1)); // print_i64
                }
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr, func)?;
                // Drop the result if not used
                func.instruction(&Instruction::Drop);
            }

            Stmt::FnDef(_, _, _) => {
                // Already handled in first pass
            }

            _ => {
                bail!("Unsupported statement type in WASM backend: {:?}", stmt);
            }
        }
        Ok(())
    }

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr, func: &mut Function) -> Result<()> {
        match expr {
            Expr::Int(n) => {
                func.instruction(&Instruction::I64Const(*n));
            }

            Expr::Float(f) => {
                func.instruction(&Instruction::F64Const(*f));
            }

            Expr::Bool(b) => {
                func.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
            }

            Expr::Ident(name) => {
                if let Some(&local_idx) = self.locals.get(name) {
                    func.instruction(&Instruction::LocalGet(local_idx));
                } else if let Some(&global_idx) = self.globals.get(name) {
                    func.instruction(&Instruction::GlobalGet(global_idx));
                } else {
                    bail!("Undefined variable: {}", name);
                }
            }

            Expr::Binary { op, left, right } => {
                self.compile_expr(left, func)?;
                self.compile_expr(right, func)?;
                
                match op {
                    BinOp::Add => { func.instruction(&Instruction::I64Add); }
                    BinOp::Sub => { func.instruction(&Instruction::I64Sub); }
                    BinOp::Mul => { func.instruction(&Instruction::I64Mul); }
                    BinOp::Div => { func.instruction(&Instruction::I64DivS); }
                    BinOp::Mod => { func.instruction(&Instruction::I64RemS); }
                    BinOp::Eq => { func.instruction(&Instruction::I64Eq); }
                    BinOp::Ne => { func.instruction(&Instruction::I64Ne); }
                    BinOp::Lt => { func.instruction(&Instruction::I64LtS); }
                    BinOp::Le => { func.instruction(&Instruction::I64LeS); }
                    BinOp::Gt => { func.instruction(&Instruction::I64GtS); }
                    BinOp::Ge => { func.instruction(&Instruction::I64GeS); }
                    BinOp::And => { func.instruction(&Instruction::I32And); }
                    BinOp::Or => { func.instruction(&Instruction::I32Or); }
                    BinOp::BitAnd => { func.instruction(&Instruction::I64And); }
                    BinOp::BitOr => { func.instruction(&Instruction::I64Or); }
                    BinOp::BitXor => { func.instruction(&Instruction::I64Xor); }
                    BinOp::Shl => { func.instruction(&Instruction::I64Shl); }
                    BinOp::Shr => { func.instruction(&Instruction::I64ShrS); }
                }
            }

            Expr::Call { callee, args } => {
                // Push arguments
                for arg in args {
                    self.compile_expr(arg, func)?;
                }

                // Call function
                if let Some(&(_, _)) = self.functions.get(callee) {
                    // User-defined function
                    let fn_idx = self.get_function_index(callee)?;
                    func.instruction(&Instruction::Call(fn_idx));
                } else {
                    bail!("Undefined function: {}", callee);
                }
            }

            Expr::UnaryNeg(expr) => {
                func.instruction(&Instruction::I64Const(0));
                self.compile_expr(expr, func)?;
                func.instruction(&Instruction::I64Sub);
            }

            Expr::UnaryNot(expr) => {
                self.compile_expr(expr, func)?;
                func.instruction(&Instruction::I32Eqz);
            }

            Expr::Array(elements) => {
                // Arrays need memory allocation - simplified for now
                // Just push first element as a placeholder
                if !elements.is_empty() {
                    self.compile_expr(&elements[0], func)?;
                } else {
                    func.instruction(&Instruction::I64Const(0));
                }
            }

            Expr::Str(s) => {
                // Store string in data section and return pointer
                let offset = self.memory_offset;
                self.string_pool.push(s.clone());
                self.memory_offset += s.len() as u32 + 1;
                func.instruction(&Instruction::I32Const(offset as i32));
            }

            _ => {
                bail!("Unsupported expression type in WASM backend: {:?}", expr);
            }
        }
        Ok(())
    }

    /// Get function index in module
    fn get_function_index(&self, name: &str) -> Result<u32> {
        // Import functions are indices 0-2 (print_i32, print_i64, print_f64)
        let import_count = 3;
        
        let mut idx = import_count;
        for (fn_name, _) in &self.functions {
            if fn_name == name {
                return Ok(idx);
            }
            idx += 1;
        }
        
        bail!("Function not found: {}", name);
    }
}

/// Compile a Tong program to WASM
pub fn compile_to_wasm(program: &Program) -> Result<Vec<u8>> {
    let mut compiler = WasmCompiler::new();
    compiler.compile_program(program)
}

/// Compile a Tong program to WAT (WebAssembly Text Format)
pub fn compile_to_wat(program: &Program) -> Result<String> {
    let wasm_bytes = compile_to_wasm(program)?;
    
    // Use wasmparser to convert binary to text format
    match wasmprinter::print_bytes(&wasm_bytes) {
        Ok(wat) => Ok(wat),
        Err(e) => bail!("Failed to convert WASM to WAT: {}", e),
    }
}
