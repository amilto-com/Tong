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