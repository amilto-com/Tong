use std::collections::HashMap;

use crate::parser::{Expr, Pattern, Stmt, TypeAnn};
use crate::value::Value;

// Clause type aliases to keep signatures and storage readable (avoid clippy::type_complexity)
type GuardedClause = (Vec<String>, Expr, Vec<Stmt>);
// Add optional return type on pattern functions (param annotations for patterns are not yet supported)
type PatternClause = (Vec<Pattern>, Option<Expr>, Option<TypeAnn>, Vec<Stmt>);

#[derive(Default)]
pub struct Env {
    pub(crate) vars_stack: Vec<HashMap<String, Value>>, // lexical-style stack
    pub(crate) muts_stack: Vec<HashMap<String, bool>>,  // per-scope mutability (true = mutable)
    pub(crate) funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    pub(crate) fn_types: HashMap<String, (Vec<Option<TypeAnn>>, Option<TypeAnn>)>, // function -> (param annotations, return)
    pub(crate) guarded_funcs: HashMap<String, Vec<GuardedClause>>,                 // guarded multi-clause
    pub(crate) pattern_funcs: HashMap<String, Vec<PatternClause>>,                 // pattern parameter clauses
    pub(crate) modules: HashMap<String, Value>,
    #[cfg(not(feature = "sdl3"))]
    pub(crate) sdl_frame: i64,
    #[cfg(feature = "sdl3")]
    pub(crate) sdl: Option<SdlState>,
    pub(crate) data_ctors: HashMap<String, usize>,       // ctor name -> arity
    pub(crate) type_ctors: HashMap<String, Vec<String>>, // type name -> ctor names
    pub(crate) ctor_type: HashMap<String, String>,       // ctor name -> type name
    pub(crate) debug: bool,
    // CLI context
    pub cli_script: Option<String>,
    pub cli_args: Vec<String>,
}

#[cfg(feature = "sdl3")]
struct SdlState {
    _sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    pub window: Option<sdl3::video::Window>,
    canvas: Option<sdl3::render::Canvas<sdl3::video::Window>>,
    events: sdl3::EventPump,
    draw_color: (u8, u8, u8, u8),
}