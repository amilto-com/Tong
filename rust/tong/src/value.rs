#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Float(f64),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
    Lambda {
        params: Vec<String>,
        body: Box<crate::parser::Expr>,
        env: std::collections::HashMap<String, Value>,
    },
    FuncRef(String),
    Object(std::collections::HashMap<String, Value>),
    Constructor {
        name: String,
        fields: Vec<Value>,
    },
    Partial {
        name: String,
        applied: Vec<Value>,
    },
    Tensor(Vec<f64>, Vec<usize>),
}

// Helper conversions for Value to numeric types (only needed when SDL backend active)
#[cfg(feature = "sdl3")]
impl Value {
    pub fn as_int_u8(&self) -> anyhow::Result<u8> {
        match self {
            Value::Int(i) => Ok((*i).clamp(0, 255) as u8),
            _ => anyhow::bail!("expected int"),
        }
    }
    pub fn as_int_u32(&self) -> anyhow::Result<u32> {
        match self {
            Value::Int(i) => Ok((*i).max(0) as u32),
            _ => anyhow::bail!("expected int"),
        }
    }
    pub fn as_int_i32(&self) -> anyhow::Result<i32> {
        match self {
            Value::Int(i) => Ok(*i as i32),
            _ => anyhow::bail!("expected int"),
        }
    }
}

pub fn format_value(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
        Value::FuncRef(name) => format!("<func:{}>", name),
        Value::Object(_) => "<object>".to_string(),
        Value::Constructor { name, fields } => {
            if fields.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}({})",
                    name,
                    fields
                        .iter()
                        .map(format_value)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        Value::Partial { name, applied } => format!("<partial:{}:{}>", name, applied.len()),
        Value::Tensor(data, dims) => format!("tensor({:?}, {:?})", data, dims),
    }
}