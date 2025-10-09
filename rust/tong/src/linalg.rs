use anyhow::{bail, Result};
use std::collections::HashMap;
use crate::value::Value;
use crate::env::Env;

impl Env {
    pub fn import_linalg(&mut self) -> Value {
        let mut obj = HashMap::new();
        // Add linalg methods as function references
        obj.insert("ones".to_string(), Value::FuncRef("linalg_ones".to_string()));
        obj.insert("zeros".to_string(), Value::FuncRef("linalg_zeros".to_string()));
        obj.insert("tensor".to_string(), Value::FuncRef("linalg_tensor".to_string()));
        obj.insert("shape".to_string(), Value::FuncRef("linalg_shape".to_string()));
        obj.insert("rank".to_string(), Value::FuncRef("linalg_rank".to_string()));
        obj.insert("get".to_string(), Value::FuncRef("linalg_get".to_string()));
        obj.insert("add".to_string(), Value::FuncRef("linalg_add".to_string()));
        obj.insert("transpose".to_string(), Value::FuncRef("linalg_transpose".to_string()));
        obj.insert("matmul".to_string(), Value::FuncRef("linalg_matmul".to_string()));
        obj.insert("dot".to_string(), Value::FuncRef("linalg_dot".to_string()));
        Value::Object(obj)
    }

    pub fn call_linalg_builtin_values(&mut self, name: &str, values: Vec<Value>) -> Result<Value> {
        match name {
            "linalg_ones" => {
                if values.len() != 1 {
                    bail!("linalg.ones expects 1 argument (shape array)");
                }
                match &values[0] {
                    Value::Array(shape) => {
                        let dims: Result<Vec<usize>> = shape.iter().map(|v| match v {
                            Value::Int(i) if *i >= 0 => Ok(*i as usize),
                            _ => bail!("shape must be array of non-negative integers"),
                        }).collect();
                        let dims = dims?;
                        let total = dims.iter().product();
                        let data = vec![1.0f64; total];
                        Ok(Value::Tensor(data, dims))
                    }
                    _ => bail!("linalg.ones expects shape as array"),
                }
            }
            "linalg_zeros" => {
                if values.len() != 1 {
                    bail!("linalg.zeros expects 1 argument (shape array)");
                }
                match &values[0] {
                    Value::Array(shape) => {
                        let dims: Result<Vec<usize>> = shape.iter().map(|v| match v {
                            Value::Int(i) if *i >= 0 => Ok(*i as usize),
                            _ => bail!("shape must be array of non-negative integers"),
                        }).collect();
                        let dims = dims?;
                        let total = dims.iter().product();
                        let data = vec![0.0f64; total];
                        Ok(Value::Tensor(data, dims))
                    }
                    _ => bail!("linalg.zeros expects shape as array"),
                }
            }
            "linalg_tensor" => {
                if values.len() != 2 {
                    bail!("linalg.tensor expects 2 arguments (data array, shape array)");
                }
                let data = match &values[0] {
                    Value::Array(arr) => {
                        let floats: Result<Vec<f64>> = arr.iter().map(|v| match v {
                            Value::Int(i) => Ok(*i as f64),
                            Value::Float(f) => Ok(*f),
                            _ => bail!("data must be array of numbers"),
                        }).collect();
                        floats?
                    }
                    _ => bail!("data must be array"),
                };
                let shape = match &values[1] {
                    Value::Array(arr) => {
                        let dims: Result<Vec<usize>> = arr.iter().map(|v| match v {
                            Value::Int(i) if *i >= 0 => Ok(*i as usize),
                            _ => bail!("shape must be array of non-negative integers"),
                        }).collect();
                        dims?
                    }
                    _ => bail!("shape must be array"),
                };
                let total: usize = shape.iter().product();
                if data.len() != total {
                    bail!("data length {} does not match shape product {}", data.len(), total);
                }
                Ok(Value::Tensor(data, shape))
            }
            "linalg_shape" => {
                if values.len() != 1 {
                    bail!("linalg.shape expects 1 argument (tensor)");
                }
                match &values[0] {
                    Value::Tensor(_, dims) => Ok(Value::Array(dims.iter().map(|&d| Value::Int(d as i64)).collect())),
                    _ => bail!("linalg.shape expects tensor"),
                }
            }
            "linalg_rank" => {
                if values.len() != 1 {
                    bail!("linalg.rank expects 1 argument (tensor)");
                }
                match &values[0] {
                    Value::Tensor(_, dims) => Ok(Value::Int(dims.len() as i64)),
                    _ => bail!("linalg.rank expects tensor"),
                }
            }
            "linalg_get" => {
                if values.len() != 2 {
                    bail!("linalg.get expects 2 arguments (tensor, indices array)");
                }
                let indices = match &values[1] {
                    Value::Array(arr) => {
                        let idx: Result<Vec<usize>> = arr.iter().map(|v| match v {
                            Value::Int(i) if *i >= 0 => Ok(*i as usize),
                            _ => bail!("indices must be array of non-negative integers"),
                        }).collect();
                        idx?
                    }
                    _ => bail!("indices must be array"),
                };
                match &values[0] {
                    Value::Tensor(data, dims) => {
                        if indices.len() != dims.len() {
                            bail!("indices length {} does not match tensor rank {}", indices.len(), dims.len());
                        }
                        let mut flat_idx = 0;
                        let mut stride = 1;
                        for (i, &idx) in indices.iter().enumerate().rev() {
                            if idx >= dims[i] {
                                bail!("index {} out of bounds for dimension {} size {}", idx, i, dims[i]);
                            }
                            flat_idx += idx * stride;
                            stride *= dims[i];
                        }
                        Ok(Value::Float(data[flat_idx]))
                    }
                    _ => bail!("linalg.get expects tensor"),
                }
            }
            "linalg_add" => {
                if values.len() != 2 {
                    bail!("linalg.add expects 2 arguments (tensor, tensor)");
                }
                match (&values[0], &values[1]) {
                    (Value::Tensor(d1, s1), Value::Tensor(d2, s2)) => {
                        if s1 != s2 {
                            bail!("tensor shapes must match for add");
                        }
                        let result: Vec<f64> = d1.iter().zip(d2.iter()).map(|(a, b)| a + b).collect();
                        Ok(Value::Tensor(result, s1.clone()))
                    }
                    _ => bail!("linalg.add expects two tensors"),
                }
            }
            "linalg_transpose" => {
                if values.len() != 1 {
                    bail!("linalg.transpose expects 1 argument (tensor)");
                }
                match &values[0] {
                    Value::Tensor(data, dims) => {
                        if dims.len() != 2 {
                            bail!("linalg.transpose expects 2D tensor");
                        }
                        let rows = dims[0];
                        let cols = dims[1];
                        let mut result = vec![0.0; data.len()];
                        for i in 0..rows {
                            for j in 0..cols {
                                result[j * rows + i] = data[i * cols + j];
                            }
                        }
                        Ok(Value::Tensor(result, vec![cols, rows]))
                    }
                    _ => bail!("linalg.transpose expects tensor"),
                }
            }
            "linalg_matmul" => {
                if values.len() != 2 {
                    bail!("linalg.matmul expects 2 arguments (tensor, tensor)");
                }
                match (&values[0], &values[1]) {
                    (Value::Tensor(d1, s1), Value::Tensor(d2, s2)) => {
                        if s1.len() != 2 || s2.len() != 2 || s1[1] != s2[0] {
                            bail!("incompatible shapes for matmul: {:?} and {:?}", s1, s2);
                        }
                        let m = s1[0];
                        let k = s1[1];
                        let n = s2[1];
                        let mut result = vec![0.0; m * n];
                        for i in 0..m {
                            for j in 0..n {
                                for p in 0..k {
                                    result[i * n + j] += d1[i * k + p] * d2[p * n + j];
                                }
                            }
                        }
                        Ok(Value::Tensor(result, vec![m, n]))
                    }
                    _ => bail!("linalg.matmul expects two tensors"),
                }
            }
            "linalg_dot" => {
                if values.len() != 2 {
                    bail!("linalg.dot expects 2 arguments (tensor, tensor)");
                }
                match (&values[0], &values[1]) {
                    (Value::Tensor(d1, s1), Value::Tensor(d2, s2)) => {
                        // Dot product for 1D tensors
                        if s1.len() != 1 || s2.len() != 1 {
                            bail!("linalg.dot expects 1D tensors");
                        }
                        if s1[0] != s2[0] {
                            bail!("incompatible shapes for dot: {:?} and {:?}", s1, s2);
                        }
                        let result: f64 = d1.iter().zip(d2.iter()).map(|(a, b)| a * b).sum();
                        Ok(Value::Float(result))
                    }
                    _ => bail!("linalg.dot expects two tensors"),
                }
            }
            _ => bail!("unknown linalg function {}", name),
        }
    }
}