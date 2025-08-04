use crate::value::Value;
use crate::interpreter::RuntimeError;
use std::collections::HashMap;

pub fn create_stdlib() -> HashMap<String, Value> {
    let mut stdlib = HashMap::new();
    
    // Mathematical functions
    stdlib.insert("abs".to_string(), Value::BuiltinFunction("abs".to_string()));
    stdlib.insert("sqrt".to_string(), Value::BuiltinFunction("sqrt".to_string()));
    stdlib.insert("pow".to_string(), Value::BuiltinFunction("pow".to_string()));
    stdlib.insert("sin".to_string(), Value::BuiltinFunction("sin".to_string()));
    stdlib.insert("cos".to_string(), Value::BuiltinFunction("cos".to_string()));
    stdlib.insert("tan".to_string(), Value::BuiltinFunction("tan".to_string()));
    stdlib.insert("floor".to_string(), Value::BuiltinFunction("floor".to_string()));
    stdlib.insert("ceil".to_string(), Value::BuiltinFunction("ceil".to_string()));
    stdlib.insert("round".to_string(), Value::BuiltinFunction("round".to_string()));
    stdlib.insert("min".to_string(), Value::BuiltinFunction("min".to_string()));
    stdlib.insert("max".to_string(), Value::BuiltinFunction("max".to_string()));
    
    // Array functions
    stdlib.insert("len".to_string(), Value::BuiltinFunction("len".to_string()));
    stdlib.insert("push".to_string(), Value::BuiltinFunction("push".to_string()));
    stdlib.insert("pop".to_string(), Value::BuiltinFunction("pop".to_string()));
    stdlib.insert("map".to_string(), Value::BuiltinFunction("map".to_string()));
    stdlib.insert("filter".to_string(), Value::BuiltinFunction("filter".to_string()));
    stdlib.insert("reduce".to_string(), Value::BuiltinFunction("reduce".to_string()));
    stdlib.insert("sum".to_string(), Value::BuiltinFunction("sum".to_string()));
    
    // String functions
    stdlib.insert("upper".to_string(), Value::BuiltinFunction("upper".to_string()));
    stdlib.insert("lower".to_string(), Value::BuiltinFunction("lower".to_string()));
    stdlib.insert("trim".to_string(), Value::BuiltinFunction("trim".to_string()));
    
    // Utility functions
    stdlib.insert("type_of".to_string(), Value::BuiltinFunction("type_of".to_string()));
    stdlib.insert("range".to_string(), Value::BuiltinFunction("range".to_string()));
    
    // Constants
    stdlib.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
    stdlib.insert("E".to_string(), Value::Float(std::f64::consts::E));
    
    stdlib
}

pub fn call_builtin_function(name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match name {
        "abs" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "abs() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(n.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(RuntimeError {
                    message: "abs() requires a number".to_string(),
                }),
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sqrt() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                _ => Err(RuntimeError {
                    message: "sqrt() requires a number".to_string(),
                }),
            }
        }
        "pow" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "pow() takes exactly 2 arguments".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(base), Value::Integer(exp)) => {
                    Ok(Value::Float((*base as f64).powf(*exp as f64)))
                }
                (Value::Float(base), Value::Integer(exp)) => {
                    Ok(Value::Float(base.powf(*exp as f64)))
                }
                (Value::Integer(base), Value::Float(exp)) => {
                    Ok(Value::Float((*base as f64).powf(*exp)))
                }
                (Value::Float(base), Value::Float(exp)) => {
                    Ok(Value::Float(base.powf(*exp)))
                }
                _ => Err(RuntimeError {
                    message: "pow() requires two numbers".to_string(),
                }),
            }
        }
        "sin" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sin() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Float((*n as f64).sin())),
                Value::Float(f) => Ok(Value::Float(f.sin())),
                _ => Err(RuntimeError {
                    message: "sin() requires a number".to_string(),
                }),
            }
        }
        "cos" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "cos() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Float((*n as f64).cos())),
                Value::Float(f) => Ok(Value::Float(f.cos())),
                _ => Err(RuntimeError {
                    message: "cos() requires a number".to_string(),
                }),
            }
        }
        "tan" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "tan() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Float((*n as f64).tan())),
                Value::Float(f) => Ok(Value::Float(f.tan())),
                _ => Err(RuntimeError {
                    message: "tan() requires a number".to_string(),
                }),
            }
        }
        "floor" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "floor() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(*n)),
                Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
                _ => Err(RuntimeError {
                    message: "floor() requires a number".to_string(),
                }),
            }
        }
        "ceil" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "ceil() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(*n)),
                Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
                _ => Err(RuntimeError {
                    message: "ceil() requires a number".to_string(),
                }),
            }
        }
        "round" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "round() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(*n)),
                Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
                _ => Err(RuntimeError {
                    message: "round() requires a number".to_string(),
                }),
            }
        }
        "min" => {
            if args.is_empty() {
                return Err(RuntimeError {
                    message: "min() requires at least 1 argument".to_string(),
                });
            }
            let mut min_val = &args[0];
            for arg in &args[1..] {
                match (min_val, arg) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b < a { min_val = arg; }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b < a { min_val = arg; }
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        if *b < (*a as f64) { min_val = arg; }
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        if (*b as f64) < *a { min_val = arg; }
                    }
                    _ => return Err(RuntimeError {
                        message: "min() requires all arguments to be numbers".to_string(),
                    }),
                }
            }
            Ok(min_val.clone())
        }
        "max" => {
            if args.is_empty() {
                return Err(RuntimeError {
                    message: "max() requires at least 1 argument".to_string(),
                });
            }
            let mut max_val = &args[0];
            for arg in &args[1..] {
                match (max_val, arg) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b > a { max_val = arg; }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b > a { max_val = arg; }
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        if *b > (*a as f64) { max_val = arg; }
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        if (*b as f64) > *a { max_val = arg; }
                    }
                    _ => return Err(RuntimeError {
                        message: "max() requires all arguments to be numbers".to_string(),
                    }),
                }
            }
            Ok(max_val.clone())
        }
        "len" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "len() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
                _ => Err(RuntimeError {
                    message: "len() requires an array or string".to_string(),
                }),
            }
        }
        "sum" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sum() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut sum = 0.0;
                    let mut is_int = true;
                    for val in arr {
                        match val {
                            Value::Integer(n) => sum += *n as f64,
                            Value::Float(f) => {
                                sum += f;
                                is_int = false;
                            }
                            _ => return Err(RuntimeError {
                                message: "sum() requires an array of numbers".to_string(),
                            }),
                        }
                    }
                    if is_int && sum.fract() == 0.0 {
                        Ok(Value::Integer(sum as i64))
                    } else {
                        Ok(Value::Float(sum))
                    }
                }
                _ => Err(RuntimeError {
                    message: "sum() requires an array".to_string(),
                }),
            }
        }
        "upper" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "upper() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_uppercase())),
                _ => Err(RuntimeError {
                    message: "upper() requires a string".to_string(),
                }),
            }
        }
        "lower" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "lower() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_lowercase())),
                _ => Err(RuntimeError {
                    message: "lower() requires a string".to_string(),
                }),
            }
        }
        "trim" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "trim() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim().to_string())),
                _ => Err(RuntimeError {
                    message: "trim() requires a string".to_string(),
                }),
            }
        }
        "type_of" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "type_of() takes exactly 1 argument".to_string(),
                });
            }
            Ok(Value::String(args[0].type_name().to_string()))
        }
        "range" => {
            match args.len() {
                1 => {
                    // range(n) -> [0, 1, 2, ..., n-1]
                    match &args[0] {
                        Value::Integer(n) => {
                            let mut result = Vec::new();
                            for i in 0..*n {
                                result.push(Value::Integer(i));
                            }
                            Ok(Value::Array(result))
                        }
                        _ => Err(RuntimeError {
                            message: "range() requires an integer".to_string(),
                        }),
                    }
                }
                2 => {
                    // range(start, end) -> [start, start+1, ..., end-1]
                    match (&args[0], &args[1]) {
                        (Value::Integer(start), Value::Integer(end)) => {
                            let mut result = Vec::new();
                            for i in *start..*end {
                                result.push(Value::Integer(i));
                            }
                            Ok(Value::Array(result))
                        }
                        _ => Err(RuntimeError {
                            message: "range() requires integers".to_string(),
                        }),
                    }
                }
                3 => {
                    // range(start, end, step) -> [start, start+step, start+2*step, ...]
                    match (&args[0], &args[1], &args[2]) {
                        (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
                            if *step == 0 {
                                return Err(RuntimeError {
                                    message: "range() step cannot be zero".to_string(),
                                });
                            }
                            let mut result = Vec::new();
                            let mut current = *start;
                            if *step > 0 {
                                while current < *end {
                                    result.push(Value::Integer(current));
                                    current += step;
                                }
                            } else {
                                while current > *end {
                                    result.push(Value::Integer(current));
                                    current += step;
                                }
                            }
                            Ok(Value::Array(result))
                        }
                        _ => Err(RuntimeError {
                            message: "range() requires integers".to_string(),
                        }),
                    }
                }
                _ => Err(RuntimeError {
                    message: "range() takes 1, 2, or 3 arguments".to_string(),
                }),
            }
        }
        _ => Err(RuntimeError {
            message: format!("Unknown builtin function: {}", name),
        }),
    }
}
