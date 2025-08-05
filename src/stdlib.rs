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
    stdlib.insert("split".to_string(), Value::BuiltinFunction("split".to_string()));
    stdlib.insert("join".to_string(), Value::BuiltinFunction("join".to_string()));
    stdlib.insert("replace".to_string(), Value::BuiltinFunction("replace".to_string()));
    
    // Utility functions
    stdlib.insert("type_of".to_string(), Value::BuiltinFunction("type_of".to_string()));
    stdlib.insert("range".to_string(), Value::BuiltinFunction("range".to_string()));
    
    // Random function
    stdlib.insert("random".to_string(), Value::BuiltinFunction("random".to_string()));
    
    // Constants
    stdlib.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
    stdlib.insert("E".to_string(), Value::Float(std::f64::consts::E));
    
    // File I/O functions
    stdlib.insert("read_file".to_string(), Value::BuiltinFunction("read_file".to_string()));
    stdlib.insert("write_file".to_string(), Value::BuiltinFunction("write_file".to_string()));
    
    // JSON functions
    stdlib.insert("parse_json".to_string(), Value::BuiltinFunction("parse_json".to_string()));
    stdlib.insert("to_json".to_string(), Value::BuiltinFunction("to_json".to_string()));
    
    // String indexing/slicing functions
    stdlib.insert("char_at".to_string(), Value::BuiltinFunction("char_at".to_string()));
    stdlib.insert("substring".to_string(), Value::BuiltinFunction("substring".to_string()));
    
    // Enhanced array functions
    stdlib.insert("sort".to_string(), Value::BuiltinFunction("sort".to_string()));
    stdlib.insert("reverse".to_string(), Value::BuiltinFunction("reverse".to_string()));
    stdlib.insert("unique".to_string(), Value::BuiltinFunction("unique".to_string()));
    stdlib.insert("contains".to_string(), Value::BuiltinFunction("contains".to_string()));
    stdlib.insert("index_of".to_string(), Value::BuiltinFunction("index_of".to_string()));
    stdlib.insert("slice".to_string(), Value::BuiltinFunction("slice".to_string()));
    stdlib.insert("find".to_string(), Value::BuiltinFunction("find".to_string()));
    
    // Enhanced string functions
    stdlib.insert("starts_with".to_string(), Value::BuiltinFunction("starts_with".to_string()));
    stdlib.insert("ends_with".to_string(), Value::BuiltinFunction("ends_with".to_string()));
    stdlib.insert("pad_left".to_string(), Value::BuiltinFunction("pad_left".to_string()));
    stdlib.insert("pad_right".to_string(), Value::BuiltinFunction("pad_right".to_string()));
    stdlib.insert("repeat".to_string(), Value::BuiltinFunction("repeat".to_string()));
    stdlib.insert("reverse_str".to_string(), Value::BuiltinFunction("reverse_str".to_string()));
    stdlib.insert("contains_str".to_string(), Value::BuiltinFunction("contains_str".to_string()));
    
    // Base conversion functions
    stdlib.insert("to_binary".to_string(), Value::BuiltinFunction("to_binary".to_string()));
    stdlib.insert("to_octal".to_string(), Value::BuiltinFunction("to_octal".to_string()));
    stdlib.insert("to_hex".to_string(), Value::BuiltinFunction("to_hex".to_string()));
    stdlib.insert("from_binary".to_string(), Value::BuiltinFunction("from_binary".to_string()));
    stdlib.insert("from_octal".to_string(), Value::BuiltinFunction("from_octal".to_string()));
    stdlib.insert("from_hex".to_string(), Value::BuiltinFunction("from_hex".to_string()));
    stdlib.insert("base_repr".to_string(), Value::BuiltinFunction("base_repr".to_string()));
    stdlib.insert("from_base".to_string(), Value::BuiltinFunction("from_base".to_string()));
    
    // Enhanced math functions
    stdlib.insert("log".to_string(), Value::BuiltinFunction("log".to_string()));
    stdlib.insert("log10".to_string(), Value::BuiltinFunction("log10".to_string()));
    stdlib.insert("exp".to_string(), Value::BuiltinFunction("exp".to_string()));
    stdlib.insert("degrees".to_string(), Value::BuiltinFunction("degrees".to_string()));
    stdlib.insert("radians".to_string(), Value::BuiltinFunction("radians".to_string()));
    stdlib.insert("clamp".to_string(), Value::BuiltinFunction("clamp".to_string()));
    
    // Date/time functions
    stdlib.insert("now".to_string(), Value::BuiltinFunction("now".to_string()));
    stdlib.insert("timestamp".to_string(), Value::BuiltinFunction("timestamp".to_string()));
    stdlib.insert("sleep".to_string(), Value::BuiltinFunction("sleep".to_string()));
    
    // Environment/system functions
    stdlib.insert("env".to_string(), Value::BuiltinFunction("env".to_string()));
    stdlib.insert("exit".to_string(), Value::BuiltinFunction("exit".to_string()));
    
    stdlib
}

pub fn call_builtin_function(name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    std::env::set_var("RUST_BACKTRACE", "full");
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
        "read_file" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "read_file() takes exactly 1 argument, the file path".to_string(),
                });
            }
            if let Value::String(path) = &args[0] {
                match std::fs::read_to_string(path) {
                    Ok(contents) => Ok(Value::String(contents)),
                    Err(_) => Err(RuntimeError {
                        message: format!("Failed to read file: {}", path),
                    }),
                }
            } else {
                Err(RuntimeError {
                    message: "read_file() requires a string file path".to_string(),
                })
            }
        }
        "write_file" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "write_file() takes exactly 2 arguments, the file path and content".to_string(),
                });
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::write(path, content) {
                    Ok(_) => Ok(Value::Nil),
                    Err(_) => Err(RuntimeError {
                        message: format!("Failed to write to file: {}", path),
                    }),
                }
            } else {
                Err(RuntimeError {
                    message: "write_file() requires a string file path and content".to_string(),
                })
            }
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
        "split" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "split() takes exactly 2 arguments (string to split and delimiter)".to_string(),
                });
            }

            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(delim)) => {
                    let parts: Vec<_> = s.split(delim.as_str()).map(|part| Value::String(part.to_string())).collect();
                    Ok(Value::Array(parts))
                }
                _ => Err(RuntimeError {
                    message: "split() requires a string and a string delimiter".to_string(),
                }),
            }
        }

        "join" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "join() takes exactly 2 arguments (array to join and delimiter)".to_string(),
                });
            }

            match (&args[0], &args[1]) {
                (Value::Array(strings), Value::String(delim)) => {
                    let joined = strings.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(delim);
                    Ok(Value::String(joined))
                }
                _ => Err(RuntimeError {
                    message: "join() requires an array of strings and a string delimiter".to_string(),
                }),
            }
        }

        "replace" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "replace() takes exactly 3 arguments (original string, pattern, replacement)".to_string(),
                });
            }

            match (&args[0], &args[1], &args[2]) {
                (Value::String(original), Value::String(pattern), Value::String(replacement)) => {
                    let replaced = original.replace(pattern, replacement);
                    Ok(Value::String(replaced))
                }
                _ => Err(RuntimeError {
                    message: "replace() requires three string arguments".to_string(),
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
        "push" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "push() takes exactly 2 arguments (array and value)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    Ok(Value::Array(new_arr))
                }
                _ => Err(RuntimeError {
                    message: "push() requires an array as first argument".to_string(),
                }),
            }
        }
        "pop" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "pop() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    if arr.is_empty() {
                        return Err(RuntimeError {
                            message: "pop() cannot pop from empty array".to_string(),
                        });
                    }
                    let mut new_arr = arr.clone();
                    let popped = new_arr.pop().unwrap();
                    Ok(popped)
                }
                _ => Err(RuntimeError {
                    message: "pop() requires an array".to_string(),
                }),
            }
        }
        "parse_json" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "parse_json() takes exactly 1 argument (JSON string)".to_string(),
                });
            }
            if let Value::String(json_str) = &args[0] {
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(json_value) => Ok(json_to_ject_value(json_value)),
                    Err(e) => Err(RuntimeError {
                        message: format!("Failed to parse JSON: {}", e),
                    }),
                }
            } else {
                Err(RuntimeError {
                    message: "parse_json() requires a string argument".to_string(),
                })
            }
        }
        "to_json" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_json() takes exactly 1 argument".to_string(),
                });
            }
            match ject_value_to_json(&args[0]) {
                Ok(json_value) => match serde_json::to_string(&json_value) {
                    Ok(json_str) => Ok(Value::String(json_str)),
                    Err(e) => Err(RuntimeError {
                        message: format!("Failed to serialize to JSON: {}", e),
                    }),
                },
                Err(e) => Err(e),
            }
        }
        "char_at" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "char_at() takes exactly 2 arguments (string and index)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Integer(idx)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let index = *idx as usize;
                    if index < chars.len() {
                        Ok(Value::String(chars[index].to_string()))
                    } else {
                        Err(RuntimeError {
                            message: "String index out of bounds".to_string(),
                        })
                    }
                }
                _ => Err(RuntimeError {
                    message: "char_at() requires a string and an integer index".to_string(),
                }),
            }
        }
        "substring" => {
            match args.len() {
                2 => {
                    // substring(string, start) - from start to end
                    match (&args[0], &args[1]) {
                        (Value::String(s), Value::Integer(start)) => {
                            let chars: Vec<char> = s.chars().collect();
                            let start_idx = *start as usize;
                            if start_idx <= chars.len() {
                                let result: String = chars[start_idx..].iter().collect();
                                Ok(Value::String(result))
                            } else {
                                Ok(Value::String(String::new()))
                            }
                        }
                        _ => Err(RuntimeError {
                            message: "substring() requires a string and integer indices".to_string(),
                        }),
                    }
                }
                3 => {
                    // substring(string, start, end)
                    match (&args[0], &args[1], &args[2]) {
                        (Value::String(s), Value::Integer(start), Value::Integer(end)) => {
                            let chars: Vec<char> = s.chars().collect();
                            let start_idx = (*start as usize).min(chars.len());
                            let end_idx = (*end as usize).min(chars.len());
                            if start_idx <= end_idx {
                                let result: String = chars[start_idx..end_idx].iter().collect();
                                Ok(Value::String(result))
                            } else {
                                Ok(Value::String(String::new()))
                            }
                        }
                        _ => Err(RuntimeError {
                            message: "substring() requires a string and integer indices".to_string(),
                        }),
                    }
                }
                _ => Err(RuntimeError {
                    message: "substring() takes 2 or 3 arguments".to_string(),
                }),
            }
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
        "random" => {
            if args.len() != 0 {
                return Err(RuntimeError {
                    message: "random() takes no arguments".to_string(),
                });
            }
            // Generate a random float between 0.0 and 1.0
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let mut hasher = DefaultHasher::new();
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            let hash = hasher.finish();
            let random_val = (hash as f64) / (u64::MAX as f64);
            Ok(Value::Float(random_val))
        }
        _ => Err(RuntimeError {
            message: format!("Unknown builtin function: {}", name),
        }),
    }
}

// Helper function to convert serde_json::Value to Ject Value
fn json_to_ject_value(json_value: serde_json::Value) -> Value {
    match json_value {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil // Fallback for weird numbers
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let ject_array: Vec<Value> = arr.into_iter().map(json_to_ject_value).collect();
            Value::Array(ject_array)
        }
        serde_json::Value::Object(obj) => {
            // Convert JSON object to Ject array of [key, value] pairs
            let pairs: Vec<Value> = obj.into_iter()
                .map(|(k, v)| Value::Array(vec![Value::String(k), json_to_ject_value(v)]))
                .collect();
            Value::Array(pairs)
        }
    }
}

// Helper function to convert Ject Value to serde_json::Value
fn ject_value_to_json(ject_value: &Value) -> Result<serde_json::Value, RuntimeError> {
    match ject_value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Ok(serde_json::Value::Number(n))
            } else {
                Err(RuntimeError {
                    message: "Invalid float value for JSON conversion".to_string(),
                })
            }
        }
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let json_array: Result<Vec<serde_json::Value>, RuntimeError> = 
                arr.iter().map(ject_value_to_json).collect();
            match json_array {
                Ok(json_arr) => Ok(serde_json::Value::Array(json_arr)),
                Err(e) => Err(e),
            }
        }
        Value::Function { .. } | Value::ModuleFunction { .. } | Value::Lambda { .. } | Value::BuiltinFunction(_) | Value::Dictionary(_) | Value::ModuleObject(_) => {
            Err(RuntimeError {
                message: "Cannot convert function to JSON".to_string(),
            })
        }
    }
}
