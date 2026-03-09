use crate::value::Value;
use crate::interpreter::RuntimeError;
use std::collections::HashMap;

/// Create CorLib - Core Library (always available)
/// These are Rust primitives that CANNOT be written in Ject itself
pub fn create_corlib() -> HashMap<String, Value> {
    let mut corlib = HashMap::new();

    // ========== Type Inspection & Conversion ==========
    // These need Rust's internal type system access
    corlib.insert("type_of".to_string(), Value::BuiltinFunction("type_of".to_string()));
    corlib.insert("to_int".to_string(), Value::BuiltinFunction("to_int".to_string()));
    corlib.insert("to_float".to_string(), Value::BuiltinFunction("to_float".to_string()));
    corlib.insert("to_string".to_string(), Value::BuiltinFunction("to_string".to_string()));
    corlib.insert("to_bool".to_string(), Value::BuiltinFunction("to_bool".to_string()));

    // ========== Collection Primitives ==========
    // These need internal access to data structures
    corlib.insert("len".to_string(), Value::BuiltinFunction("len".to_string()));
    corlib.insert("range".to_string(), Value::BuiltinFunction("range".to_string()));
    corlib.insert("push".to_string(), Value::BuiltinFunction("push".to_string()));
    corlib.insert("pop".to_string(), Value::BuiltinFunction("pop".to_string()));

    // ========== Array Primitives ==========
    // These need internal array access
    corlib.insert("sum".to_string(), Value::BuiltinFunction("sum".to_string()));
    corlib.insert("contains".to_string(), Value::BuiltinFunction("contains".to_string()));
    corlib.insert("index_of".to_string(), Value::BuiltinFunction("index_of".to_string()));
    corlib.insert("first".to_string(), Value::BuiltinFunction("first".to_string()));
    corlib.insert("last".to_string(), Value::BuiltinFunction("last".to_string()));
    corlib.insert("sort".to_string(), Value::BuiltinFunction("sort".to_string()));
    corlib.insert("reverse".to_string(), Value::BuiltinFunction("reverse".to_string()));
    corlib.insert("unique".to_string(), Value::BuiltinFunction("unique".to_string()));

    // ========== Higher-Order Functions ==========
    // These execute lambdas - need Rust interpreter access
    corlib.insert("map".to_string(), Value::BuiltinFunction("map".to_string()));
    corlib.insert("filter".to_string(), Value::BuiltinFunction("filter".to_string()));
    corlib.insert("reduce".to_string(), Value::BuiltinFunction("reduce".to_string()));

    // ========== Math Primitives ==========
    // These use Rust's std::f64 math - cannot be written in Ject
    corlib.insert("abs".to_string(), Value::BuiltinFunction("abs".to_string()));
    corlib.insert("sqrt".to_string(), Value::BuiltinFunction("sqrt".to_string()));
    corlib.insert("pow".to_string(), Value::BuiltinFunction("pow".to_string()));
    corlib.insert("sin".to_string(), Value::BuiltinFunction("sin".to_string()));
    corlib.insert("cos".to_string(), Value::BuiltinFunction("cos".to_string()));
    corlib.insert("tan".to_string(), Value::BuiltinFunction("tan".to_string()));
    corlib.insert("floor".to_string(), Value::BuiltinFunction("floor".to_string()));
    corlib.insert("ceil".to_string(), Value::BuiltinFunction("ceil".to_string()));
    corlib.insert("round".to_string(), Value::BuiltinFunction("round".to_string()));
    corlib.insert("min".to_string(), Value::BuiltinFunction("min".to_string()));
    corlib.insert("max".to_string(), Value::BuiltinFunction("max".to_string()));
    corlib.insert("random".to_string(), Value::BuiltinFunction("random".to_string()));
    corlib.insert("random_int".to_string(), Value::BuiltinFunction("random_int".to_string()));

    // ========== String Primitives ==========
    // These need internal string access
    corlib.insert("upper".to_string(), Value::BuiltinFunction("upper".to_string()));
    corlib.insert("lower".to_string(), Value::BuiltinFunction("lower".to_string()));
    corlib.insert("trim".to_string(), Value::BuiltinFunction("trim".to_string()));
    corlib.insert("split".to_string(), Value::BuiltinFunction("split".to_string()));
    corlib.insert("join".to_string(), Value::BuiltinFunction("join".to_string()));
    corlib.insert("replace".to_string(), Value::BuiltinFunction("replace".to_string()));
    corlib.insert("char_at".to_string(), Value::BuiltinFunction("char_at".to_string()));
    corlib.insert("substring".to_string(), Value::BuiltinFunction("substring".to_string()));
    corlib.insert("repeat".to_string(), Value::BuiltinFunction("repeat".to_string()));

    // ========== I/O Primitives ==========
    // These need system access
    corlib.insert("input".to_string(), Value::BuiltinFunction("input".to_string()));
    corlib.insert("print".to_string(), Value::BuiltinFunction("print".to_string()));
    corlib.insert("read_file".to_string(), Value::BuiltinFunction("read_file".to_string()));
    corlib.insert("write_file".to_string(), Value::BuiltinFunction("write_file".to_string()));

    // ========== Testing ==========
    corlib.insert("assert".to_string(), Value::BuiltinFunction("assert".to_string()));

    // ========== Constants ==========
    corlib.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
    corlib.insert("E".to_string(), Value::Float(std::f64::consts::E));

    corlib
}

/// Get math module functions (import "math")
/// Advanced math functions NOT in CorLib - written in Ject
pub fn get_math_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();

    // Advanced math (NOT in CorLib - written in Ject)
    module.insert("log".to_string(), Value::BuiltinFunction("log".to_string()));
    module.insert("log10".to_string(), Value::BuiltinFunction("log10".to_string()));
    module.insert("exp".to_string(), Value::BuiltinFunction("exp".to_string()));
    module.insert("log2".to_string(), Value::BuiltinFunction("log2".to_string()));
    module.insert("ln".to_string(), Value::BuiltinFunction("ln".to_string()));

    // Angle conversion
    module.insert("degrees".to_string(), Value::BuiltinFunction("degrees".to_string()));
    module.insert("radians".to_string(), Value::BuiltinFunction("radians".to_string()));
    module.insert("deg_to_rad".to_string(), Value::BuiltinFunction("deg_to_rad".to_string()));
    module.insert("rad_to_deg".to_string(), Value::BuiltinFunction("rad_to_deg".to_string()));

    // Advanced functions
    module.insert("clamp".to_string(), Value::BuiltinFunction("clamp".to_string()));
    module.insert("sign".to_string(), Value::BuiltinFunction("sign".to_string()));
    module.insert("gcd".to_string(), Value::BuiltinFunction("gcd".to_string()));
    module.insert("lcm".to_string(), Value::BuiltinFunction("lcm".to_string()));

    // Inverse trig (not in CorLib)
    module.insert("asin".to_string(), Value::BuiltinFunction("asin".to_string()));
    module.insert("acos".to_string(), Value::BuiltinFunction("acos".to_string()));
    module.insert("atan".to_string(), Value::BuiltinFunction("atan".to_string()));
    module.insert("atan2".to_string(), Value::BuiltinFunction("atan2".to_string()));

    // Hyperbolic (not in CorLib)
    module.insert("sinh".to_string(), Value::BuiltinFunction("sinh".to_string()));
    module.insert("cosh".to_string(), Value::BuiltinFunction("cosh".to_string()));
    module.insert("tanh".to_string(), Value::BuiltinFunction("tanh".to_string()));

    // Rounding variants
    module.insert("round_to".to_string(), Value::BuiltinFunction("round_to".to_string()));

    module
}

/// Get string module functions (import "string")
/// Advanced string functions NOT in CorLib - written in Ject
pub fn get_string_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();

    // Advanced case conversion (NOT in CorLib)
    module.insert("capitalize".to_string(), Value::BuiltinFunction("capitalize".to_string()));
    module.insert("title_case".to_string(), Value::BuiltinFunction("title_case".to_string()));

    // Trimming variants
    module.insert("trim_left".to_string(), Value::BuiltinFunction("trim_left".to_string()));
    module.insert("trim_right".to_string(), Value::BuiltinFunction("trim_right".to_string()));

    // Padding
    module.insert("pad_left".to_string(), Value::BuiltinFunction("pad_left".to_string()));
    module.insert("pad_right".to_string(), Value::BuiltinFunction("pad_right".to_string()));
    module.insert("pad_center".to_string(), Value::BuiltinFunction("pad_center".to_string()));

    // Search and test (NOT in CorLib)
    module.insert("starts_with".to_string(), Value::BuiltinFunction("starts_with".to_string()));
    module.insert("ends_with".to_string(), Value::BuiltinFunction("ends_with".to_string()));
    module.insert("contains_str".to_string(), Value::BuiltinFunction("contains_str".to_string()));
    module.insert("count".to_string(), Value::BuiltinFunction("count".to_string()));
    module.insert("find".to_string(), Value::BuiltinFunction("find".to_string()));

    // Manipulation variants
    module.insert("replace_all".to_string(), Value::BuiltinFunction("replace_all".to_string()));
    module.insert("replace_first".to_string(), Value::BuiltinFunction("replace_first".to_string()));
    module.insert("remove".to_string(), Value::BuiltinFunction("remove".to_string()));
    module.insert("repeat".to_string(), Value::BuiltinFunction("repeat".to_string()));
    module.insert("reverse_str".to_string(), Value::BuiltinFunction("reverse_str".to_string()));

    // Extraction
    module.insert("left".to_string(), Value::BuiltinFunction("left".to_string()));
    module.insert("right".to_string(), Value::BuiltinFunction("right".to_string()));
    module.insert("truncate".to_string(), Value::BuiltinFunction("truncate".to_string()));

    // Analysis
    module.insert("is_empty".to_string(), Value::BuiltinFunction("is_empty".to_string()));
    module.insert("is_numeric".to_string(), Value::BuiltinFunction("is_numeric".to_string()));
    module.insert("is_alpha".to_string(), Value::BuiltinFunction("is_alpha".to_string()));
    module.insert("is_alphanumeric".to_string(), Value::BuiltinFunction("is_alphanumeric".to_string()));
    module.insert("word_count".to_string(), Value::BuiltinFunction("word_count".to_string()));
    module.insert("sentence_count".to_string(), Value::BuiltinFunction("sentence_count".to_string()));
    module.insert("paragraph_count".to_string(), Value::BuiltinFunction("paragraph_count".to_string()));
    module.insert("lines".to_string(), Value::BuiltinFunction("lines".to_string()));

    // Conversion utilities
    module.insert("extract_numbers".to_string(), Value::BuiltinFunction("extract_numbers".to_string()));
    module.insert("to_char_codes".to_string(), Value::BuiltinFunction("to_char_codes".to_string()));
    module.insert("from_char_codes".to_string(), Value::BuiltinFunction("from_char_codes".to_string()));

    // Formatting
    module.insert("format".to_string(), Value::BuiltinFunction("format".to_string()));
    module.insert("escape".to_string(), Value::BuiltinFunction("escape".to_string()));
    module.insert("unescape".to_string(), Value::BuiltinFunction("unescape".to_string()));
    module.insert("wrap_text".to_string(), Value::BuiltinFunction("wrap_text".to_string()));

    module
}

/// Get array module functions (import "array")
/// Advanced array functions NOT in CorLib - written in Ject
pub fn get_array_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();

    // Aggregation (NOT in CorLib)
    module.insert("any".to_string(), Value::BuiltinFunction("any".to_string()));
    module.insert("all".to_string(), Value::BuiltinFunction("all".to_string()));
    module.insert("average".to_string(), Value::BuiltinFunction("average".to_string()));
    module.insert("median".to_string(), Value::BuiltinFunction("median".to_string()));

    // Search variants (NOT in CorLib)
    module.insert("find".to_string(), Value::BuiltinFunction("find".to_string()));
    module.insert("count".to_string(), Value::BuiltinFunction("count".to_string()));

    // Slicing variants
    module.insert("slice".to_string(), Value::BuiltinFunction("slice".to_string()));
    module.insert("take".to_string(), Value::BuiltinFunction("take".to_string()));
    module.insert("drop".to_string(), Value::BuiltinFunction("drop".to_string()));
    module.insert("initial".to_string(), Value::BuiltinFunction("initial".to_string()));
    module.insert("rest".to_string(), Value::BuiltinFunction("rest".to_string()));

    // Combination (NOT in CorLib)
    module.insert("concat".to_string(), Value::BuiltinFunction("concat".to_string()));
    module.insert("zip".to_string(), Value::BuiltinFunction("zip".to_string()));
    module.insert("union".to_string(), Value::BuiltinFunction("union".to_string()));
    module.insert("intersection".to_string(), Value::BuiltinFunction("intersection".to_string()));
    module.insert("difference".to_string(), Value::BuiltinFunction("difference".to_string()));

    // Transformation (NOT in CorLib)
    module.insert("flatten".to_string(), Value::BuiltinFunction("flatten".to_string()));
    module.insert("chunk".to_string(), Value::BuiltinFunction("chunk".to_string()));
    module.insert("group_by".to_string(), Value::BuiltinFunction("group_by".to_string()));
    module.insert("partition".to_string(), Value::BuiltinFunction("partition".to_string()));
    module.insert("shuffle".to_string(), Value::BuiltinFunction("shuffle".to_string()));
    module.insert("rotate_left".to_string(), Value::BuiltinFunction("rotate_left".to_string()));
    module.insert("rotate_right".to_string(), Value::BuiltinFunction("rotate_right".to_string()));

    // Modification
    module.insert("insert_at".to_string(), Value::BuiltinFunction("insert_at".to_string()));
    module.insert("remove_at".to_string(), Value::BuiltinFunction("remove_at".to_string()));
    module.insert("without".to_string(), Value::BuiltinFunction("without".to_string()));
    module.insert("compact".to_string(), Value::BuiltinFunction("compact".to_string()));
    module.insert("compact_unique".to_string(), Value::BuiltinFunction("compact_unique".to_string()));

    // Utilities
    module.insert("enumerate".to_string(), Value::BuiltinFunction("enumerate".to_string()));
    module.insert("fill".to_string(), Value::BuiltinFunction("fill".to_string()));
    module.insert("range_arr".to_string(), Value::BuiltinFunction("range_arr".to_string()));
    module.insert("sample".to_string(), Value::BuiltinFunction("sample".to_string()));
    module.insert("sort_by".to_string(), Value::BuiltinFunction("sort_by".to_string()));
    module.insert("to_uarray".to_string(), Value::BuiltinFunction("to_uarray".to_string()));
    module.insert("to_array".to_string(), Value::BuiltinFunction("to_array".to_string()));

    module
}

/// Get IO module functions (import "io")
pub fn get_io_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();
    
    module.insert("read_file".to_string(), Value::BuiltinFunction("read_file".to_string()));
    module.insert("write_file".to_string(), Value::BuiltinFunction("write_file".to_string()));
    
    module
}

/// Get JSON module functions (import "json")
pub fn get_json_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();
    
    module.insert("parse_json".to_string(), Value::BuiltinFunction("parse_json".to_string()));
    module.insert("to_json".to_string(), Value::BuiltinFunction("to_json".to_string()));
    
    module
}

/// Get system module functions (import "system")
pub fn get_system_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();
    
    module.insert("env".to_string(), Value::BuiltinFunction("env".to_string()));
    module.insert("exit".to_string(), Value::BuiltinFunction("exit".to_string()));
    module.insert("now".to_string(), Value::BuiltinFunction("now".to_string()));
    module.insert("timestamp".to_string(), Value::BuiltinFunction("timestamp".to_string()));
    module.insert("sleep".to_string(), Value::BuiltinFunction("sleep".to_string()));
    
    module
}

/// Get base conversion module (import "base")
pub fn get_base_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();
    
    module.insert("to_binary".to_string(), Value::BuiltinFunction("to_binary".to_string()));
    module.insert("to_octal".to_string(), Value::BuiltinFunction("to_octal".to_string()));
    module.insert("to_hex".to_string(), Value::BuiltinFunction("to_hex".to_string()));
    module.insert("from_binary".to_string(), Value::BuiltinFunction("from_binary".to_string()));
    module.insert("from_octal".to_string(), Value::BuiltinFunction("from_octal".to_string()));
    module.insert("from_hex".to_string(), Value::BuiltinFunction("from_hex".to_string()));
    module.insert("base_repr".to_string(), Value::BuiltinFunction("base_repr".to_string()));
    module.insert("from_base".to_string(), Value::BuiltinFunction("from_base".to_string()));
    
    module
}

/// Get a module by name (for import system)
/// Returns None for modules that exist as .ject files (they will be loaded from disk)
/// Only returns Some() for Rust-only modules that don't have .ject equivalents
pub fn get_module(name: &str) -> Option<HashMap<String, Value>> {
    match name {
        // These modules exist as .ject files in stdlib/ - load from disk
        // "math", "string", "array", "io", "json", "system" - all load from stdlib/*.ject

        // Rust-only modules (no .ject equivalent)
        "base" => Some(get_base_module()),
        "numpy" => Some(crate::numpy::create_numpy_module()),

        // All other modules will be loaded from .ject files
        _ => None,
    }
}

/// Create the full StdLib (CorLib + all modules for backward compatibility)
/// DEPRECATED: New code should use modules via import instead
/// Note: This only includes CorLib + Rust-only modules (like "base")
/// Modules that exist as .ject files (math, string, array, etc.) are NOT included here
pub fn create_stdlib() -> HashMap<String, Value> {
    let mut stdlib = create_corlib();

    // Only include Rust-only modules (no .ject equivalent)
    // "base" module has no .ject file, so include it
    stdlib.extend(get_base_module());

    // Random (utility) - keep for backward compatibility
    stdlib.insert("random".to_string(), Value::BuiltinFunction("random".to_string()));

    stdlib
}

pub fn call_builtin_function(name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    std::env::set_var("RUST_BACKTRACE", "full");
match name {
        // Enhanced array functions
        "sort" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sort() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    Ok(Value::Array(new_arr))
                }
                _ => Err(RuntimeError {
                    message: "sort() requires an array".to_string(),
                }),
            }
        },
        "reverse" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "reverse() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.reverse();
                    Ok(Value::Array(new_arr))
                }
                _ => Err(RuntimeError {
                    message: "reverse() requires an array".to_string(),
                }),
            }
        },

        // Enhanced string functions
        "starts_with" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "startswith() takes exactly 2 arguments (string, prefix)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err(RuntimeError {
                    message: "startswith() requires two strings".to_string(),
                }),
            }
        },
        "ends_with" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "endswith() takes exactly 2 arguments (string, suffix)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err(RuntimeError {
                    message: "endswith() requires two strings".to_string(),
                }),
            }
        },

        // Base conversion functions
        "to_binary" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_binary() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::String(format!("{:b}", n))),
                _ => Err(RuntimeError {
                    message: "to_binary() requires an integer".to_string(),
                }),
            }
        },
        "from_binary" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "from_binary() takes exactly 1 argument (binary string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => match i64::from_str_radix(s, 2) {
                    Ok(num) => Ok(Value::Integer(num)),
                    Err(_) => Err(RuntimeError {
                        message: "Invalid binary string".to_string(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "from_binary() requires a binary string".to_string(),
                }),
            }
        },

        // Enhanced math functions
        "log" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "log() takes exactly 2 arguments (value, base)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Float(n), Value::Float(base)) => {
                    if *n <= 0.0 || *base <= 0.0 {
                        return Err(RuntimeError {
                            message: "log() requires positive number and base".to_string(),
                        });
                    }
                    Ok(Value::Float(n.log(*base)))
                },
                _ => Err(RuntimeError {
                    message: "log() requires two numbers".to_string(),
                }),
            }
        },
        "exp" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "exp() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Float(n) => Ok(Value::Float(n.exp())),
                _ => Err(RuntimeError {
                    message: "exp() requires a number".to_string(),
                }),
            }
        },

        // Date/time functions
        "now" => {
            if !args.is_empty() {
                return Err(RuntimeError {
                    message: "now() takes no arguments".to_string(),
                });
            }
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            Ok(Value::Integer(now as i64))
        },

        // Environment/system functions
        "env" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "env() takes exactly 1 argument (variable name)".to_string(),
                });
            }
            match &args[0] {
                Value::String(var) => match std::env::var(var) {
                    Ok(val) => Ok(Value::String(val)),
                    Err(_) => Ok(Value::Nil),
                },
                _ => Err(RuntimeError {
                    message: "env() requires a string variable name".to_string(),
                }),
            }
        },
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
        },
        "sqrt" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sqrt() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => {
                    if *n < 0 {
                        return Err(RuntimeError {
                            message: "sqrt() of negative number is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float((*n as f64).sqrt()))
                }
                Value::Float(f) => {
                    if *f < 0.0 {
                        return Err(RuntimeError {
                            message: "sqrt() of negative number is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float(f.sqrt()))
                }
                _ => Err(RuntimeError {
                    message: "sqrt() requires a number".to_string(),
                }),
            }
        },
        "pow" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "pow() takes exactly 2 arguments".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(base), Value::Integer(exp)) => {
                    // Check for 0^0
                    if *base == 0 && *exp == 0 {
                        return Err(RuntimeError {
                            message: "pow(): 0^0 is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float((*base as f64).powf(*exp as f64)))
                }
                (Value::Float(base), Value::Integer(exp)) => {
                    // Check for 0^0
                    if *base == 0.0 && *exp == 0 {
                        return Err(RuntimeError {
                            message: "pow(): 0^0 is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float(base.powf(*exp as f64)))
                }
                (Value::Integer(base), Value::Float(exp)) => {
                    // Check for 0^0
                    if *base == 0 && *exp == 0.0 {
                        return Err(RuntimeError {
                            message: "pow(): 0^0 is undefined".to_string(),
                        });
                    }
                    // Check for negative base with non-integer exponent
                    if *base < 0 && exp.fract() != 0.0 {
                        return Err(RuntimeError {
                            message: "pow(): negative base with non-integer exponent is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float((*base as f64).powf(*exp)))
                }
                (Value::Float(base), Value::Float(exp)) => {
                    // Check for 0^0
                    if *base == 0.0 && *exp == 0.0 {
                        return Err(RuntimeError {
                            message: "pow(): 0^0 is undefined".to_string(),
                        });
                    }
                    // Check for negative base with non-integer exponent
                    if *base < 0.0 && exp.fract() != 0.0 {
                        return Err(RuntimeError {
                            message: "pow(): negative base with non-integer exponent is undefined".to_string(),
                        });
                    }
                    Ok(Value::Float(base.powf(*exp)))
                }
                _ => Err(RuntimeError {
                    message: "pow() requires two numbers".to_string(),
                }),
            }
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
        "map" => {
            // map is implemented specially in the interpreter to support lambdas
            // This is a fallback that just returns the array
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "map() takes exactly 2 arguments (array, function)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => Ok(Value::Array(arr.clone())),
                _ => Err(RuntimeError {
                    message: "map() requires an array".to_string(),
                }),
            }
        },
        "filter" => {
            // filter is implemented specially in the interpreter to support lambdas
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "filter() takes exactly 2 arguments (array, function)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => Ok(Value::Array(arr.clone())),
                _ => Err(RuntimeError {
                    message: "filter() requires an array".to_string(),
                }),
            }
        },
        "reduce" => {
            // reduce is implemented specially in the interpreter to support lambdas
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "reduce() takes exactly 3 arguments (array, function, initial)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(_arr) => Ok(args[2].clone()),
                _ => Err(RuntimeError {
                    message: "reduce() requires an array".to_string(),
                }),
            }
        },
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
        },
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
        },
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
        },
        "split" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "split() takes exactly 2 arguments (string to split and delimiter)".to_string(),
                });
            }

            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(delim)) => {
                    let parts: Vec<_> = if delim.is_empty() {
                        // Split into individual characters
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    } else {
                        s.split(delim.as_str()).map(|part| Value::String(part.to_string())).collect()
                    };
                    Ok(Value::Array(parts))
                }
                _ => Err(RuntimeError {
                    message: "split() requires a string and a string delimiter".to_string(),
                }),
            }
        },
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
        },
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
        },
        "repeat" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "repeat() takes exactly 2 arguments (string and count)".to_string(),
                });
            }

            match (&args[0], &args[1]) {
                (Value::String(s), Value::Integer(n)) => {
                    if *n < 0 {
                        return Err(RuntimeError {
                            message: "repeat() count cannot be negative".to_string(),
                        });
                    }
                    let repeated = s.repeat(*n as usize);
                    Ok(Value::String(repeated))
                }
                _ => Err(RuntimeError {
                    message: "repeat() requires a string and an integer".to_string(),
                }),
            }
        },
        "type_of" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "type_of() takes exactly 1 argument".to_string(),
                });
            }
            Ok(Value::String(args[0].type_name().to_string()))
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
        
        // Array functions
        "first" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "first() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    if arr.is_empty() {
                        Ok(Value::Nil)
                    } else {
                        Ok(arr[0].clone())
                    }
                }
                _ => Err(RuntimeError {
                    message: "first() requires an array".to_string(),
                }),
            }
        },
        "last" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "last() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    if arr.is_empty() {
                        Ok(Value::Nil)
                    } else {
                        Ok(arr[arr.len() - 1].clone())
                    }
                }
                _ => Err(RuntimeError {
                    message: "last() requires an array".to_string(),
                }),
            }
        },
"take" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "take() takes exactly 2 arguments (array, count)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr), Value::Integer(n)) => {
                    let count = (*n as usize).min(arr.len());
                    Ok(Value::Array(arr[..count].to_vec()))
                }
                _ => Err(RuntimeError {
                    message: "take() requires an array and an integer".to_string(),
                }),
            }
        },
        "zip" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "zip() takes exactly 2 arguments (array1, array2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr1), Value::Array(arr2)) => {
                    let min_len = arr1.len().min(arr2.len());
                    let zipped: Vec<Value> = arr1.iter().zip(arr2.iter()).take(min_len).map(|(a, b)| Value::Array(vec![a.clone(), b.clone()])).collect();
                    Ok(Value::Array(zipped))
                }
                _ => Err(RuntimeError {
                    message: "zip() requires two arrays".to_string(),
                }),
            }
        },
        "enumerate" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "enumerate() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let enumerated: Vec<Value> = arr.iter().enumerate().map(|(i, v)| Value::Array(vec![Value::Integer(i as i64), v.clone()])).collect();
                    Ok(Value::Array(enumerated))
                }
                _ => Err(RuntimeError {
                    message: "enumerate() requires an array".to_string(),
                }),
            }
        },
        "unique" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "unique() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut seen = Vec::new();
                    let mut unique = Vec::new();
                    for item in arr {
                        if !seen.contains(item) {
                            seen.push(item.clone());
                            unique.push(item.clone());
                        }
                    }
                    Ok(Value::Array(unique))
                }
                _ => Err(RuntimeError {
                    message: "unique() requires an array".to_string(),
                }),
            }
        },
        "to_uarray" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_uarray() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    // Convert array to unique array (deduplicate)
                    let mut seen = std::collections::HashSet::new();
                    let mut unique = Vec::new();
                    for item in arr {
                        let key = item.to_string();
                        if !seen.contains(&key) {
                            seen.insert(key);
                            unique.push(item.clone());
                        }
                    }
                    Ok(Value::UniqueArray(unique))
                }
                Value::UniqueArray(uarr) => {
                    // Already a unique array, return copy
                    Ok(Value::UniqueArray(uarr.clone()))
                }
                _ => Err(RuntimeError {
                    message: "to_uarray() requires an array".to_string(),
                }),
            }
        },
        "contains" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "contains() takes exactly 2 arguments (array, value)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr), value) => Ok(Value::Bool(arr.contains(value))),
                _ => Err(RuntimeError {
                    message: "contains() requires an array and a value".to_string(),
                }),
            }
        },
        "index_of" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "index_of() takes exactly 2 arguments (array, value)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr), value) => Ok(Value::Integer(arr.iter().position(|x| x == value).map_or(-1, |i| i as i64))),
                _ => Err(RuntimeError {
                    message: "index_of() requires an array and a value".to_string(),
                }),
            }
        },
        "slice" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "slice() takes exactly 3 arguments (array, start, end)".to_string(),
                });
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::Array(arr), Value::Integer(start), Value::Integer(end)) => {
                    let start = *start as usize;
                    let end = (*end as usize).min(arr.len());
                    if start <= end {
                        Ok(Value::Array(arr[start..end].to_vec()))
                    } else {
                        Err(RuntimeError {
                            message: "slice() start index must be less than or equal to end index".to_string(),
                        })
                    }
                }
                _ => Err(RuntimeError {
                    message: "slice() requires an array and two integers".to_string(),
                }),
            }
        },
        "title_case" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "title_case() takes exactly 1 argument (string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => {
                    let titled: String = s.split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            if let Some(first) = chars.next() {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            } else {
                                String::new()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    Ok(Value::String(titled))
                }
                _ => Err(RuntimeError {
                    message: "title_case() requires a string".to_string(),
                }),
            }
        },
        "count" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "count() takes exactly 2 arguments (string, substring)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(sub)) => {
                    Ok(Value::Integer(s.matches(sub).count() as i64))
                }
                _ => Err(RuntimeError {
                    message: "count() requires two strings".to_string(),
                }),
            }
        },
        "lines" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "lines() takes exactly 1 argument (string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => {
                    let lines: Vec<Value> = s.lines().map(|line| Value::String(line.to_string())).collect();
                    Ok(Value::Array(lines))
                }
                _ => Err(RuntimeError {
                    message: "lines() requires a string".to_string(),
                }),
            }
        },
        "gcd" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "gcd() takes exactly 2 arguments (int, int)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    fn gcd(mut n: i64, mut m: i64) -> i64 {
                        while m != 0 {
                            let temp = m;
                            m = n % m;
                            n = temp;
                        }
                        n.abs()
                    }
                    Ok(Value::Integer(gcd(*a, *b)))
                }
                _ => Err(RuntimeError {
                    message: "gcd() requires two integers".to_string(),
                }),
            }
        },
        "lcm" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "lcm() takes exactly 2 arguments (int, int)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    fn lcm(n: i64, m: i64) -> i64 {
                        (n * m / gcd(n, m)).abs()
                    }
                    fn gcd(mut n: i64, mut m: i64) -> i64 {
                        while m != 0 {
                            let temp = m;
                            m = n % m;
                            n = temp;
                        }
                        n.abs()
                    }
                    Ok(Value::Integer(lcm(*a, *b)))
                }
                _ => Err(RuntimeError {
                    message: "lcm() requires two integers".to_string(),
                }),
            }
        },
        "is_prime" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_prime() takes exactly 1 argument (int)".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => {
                    if *n <= 1 {
                        return Ok(Value::Bool(false));
                    }
                    for i in 2..=((*n as f64).sqrt() as i64) {
                        if *n % i == 0 {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err(RuntimeError {
                    message: "is_prime() requires an integer".to_string(),
                }),
            }
        },
        "to_octal" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_octal() takes exactly 1 argument (int)".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::String(format!("{:o}", n))),
                _ => Err(RuntimeError {
                    message: "to_octal() requires an integer".to_string(),
                }),
            }
        },
        "from_octal" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "from_octal() takes exactly 1 argument (string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => match i64::from_str_radix(s, 8) {
                    Ok(num) => Ok(Value::Integer(num)),
                    Err(_) => Err(RuntimeError {
                        message: "Invalid octal string".to_string(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "from_octal() requires a string".to_string(),
                }),
            }
        },
        "to_hex" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_hex() takes exactly 1 argument (int)".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::String(format!("{:x}", n))),
                _ => Err(RuntimeError {
                    message: "to_hex() requires an integer".to_string(),
                }),
            }
        },
        "from_hex" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "from_hex() takes exactly 1 argument (string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => match i64::from_str_radix(s, 16) {
                    Ok(num) => Ok(Value::Integer(num)),
                    Err(_) => Err(RuntimeError {
                        message: "Invalid hexadecimal string".to_string(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "from_hex() requires a string".to_string(),
                }),
            }
        },
        "base_repr" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "base_repr() takes exactly 2 arguments (int, base)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(n), Value::Integer(base)) => {
                    if !(*base == 2 || *base == 8 || *base == 10 || *base == 16) {
                        return Err(RuntimeError {
                            message: "base_repr() supports only base 2, 8, 10, or 16".to_string(),
                        });
                    }
                    let representation = match *base {
                        2 => format!("{:b}", n),
                        8 => format!("{:o}", n),
                        10 => n.to_string(),
                        16 => format!("{:x}", n),
                        _ => unreachable!(),
                    };
                    Ok(Value::String(representation))
                }
                _ => Err(RuntimeError {
                    message: "base_repr() requires an integer and a base".to_string(),
                }),
            }
        },
        "from_base" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "from_base() takes exactly 2 arguments (string, base)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Integer(base)) => match i64::from_str_radix(s, *base as u32) {
                    Ok(num) => Ok(Value::Integer(num)),
                    Err(_) => Err(RuntimeError {
                        message: format!("Invalid string for base {}", base).to_string(),
                    }),
                },
                _ => Err(RuntimeError {
                    message: "from_base() requires a string and a base".to_string(),
                }),
            }
        },
        "drop" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "drop() takes exactly 2 arguments (array, count)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr), Value::Integer(n)) => {
                    let count = (*n as usize).min(arr.len());
                    Ok(Value::Array(arr[count..].to_vec()))
                }
                _ => Err(RuntimeError {
                    message: "drop() requires an array and an integer".to_string(),
                }),
            }
        },
        "concat" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "concat() takes exactly 2 arguments (array, array)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Array(arr1), Value::Array(arr2)) => {
                    let mut result = arr1.clone();
                    result.extend(arr2.clone());
                    Ok(Value::Array(result))
                }
                _ => Err(RuntimeError {
                    message: "concat() requires two arrays".to_string(),
                }),
            }
        },
        "flatten" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "flatten() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    let mut result = Vec::new();
                    for item in arr {
                        match item {
                            Value::Array(inner) => result.extend(inner.clone()),
                            _ => result.push(item.clone()),
                        }
                    }
                    Ok(Value::Array(result))
                }
                _ => Err(RuntimeError {
                    message: "flatten() requires an array".to_string(),
                }),
            }
        },
        
        // String functions
        "capitalize" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "capitalize() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => {
                    let mut chars: Vec<char> = s.chars().collect();
                    if !chars.is_empty() {
                        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                        for i in 1..chars.len() {
                            chars[i] = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                        }
                    }
                    Ok(Value::String(chars.into_iter().collect()))
                }
                _ => Err(RuntimeError {
                    message: "capitalize() requires a string".to_string(),
                }),
            }
        },
        "is_empty" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_empty() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Bool(s.is_empty())),
                Value::Array(arr) => Ok(Value::Bool(arr.is_empty())),
                _ => Ok(Value::Bool(false)),
            }
        },
        "is_numeric" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_numeric() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Bool(s.parse::<f64>().is_ok())),
                Value::Integer(_) | Value::Float(_) => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(false)),
            }
        },
        "is_alpha" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_alpha() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic()))),
                _ => Ok(Value::Bool(false)),
            }
        },
        
        // Type conversion functions
        "to_int" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_int() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(*n)),
                Value::Float(f) => Ok(Value::Integer(*f as i64)),
                Value::String(s) => {
                    // Trim whitespace and try parsing
                    let trimmed = s.trim();
                    // Try parsing as integer first
                    if let Ok(n) = trimmed.parse::<i64>() {
                        Ok(Value::Integer(n))
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        // If it's a float string, floor it
                        Ok(Value::Integer(f.floor() as i64))
                    } else {
                        Err(RuntimeError {
                            message: format!("Cannot convert '{}' to integer", s),
                        })
                    }
                },
                Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
                _ => Err(RuntimeError {
                    message: format!("Cannot convert {} to integer", args[0].type_name()),
                }),
            }
        },
        "to_float" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_float() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Float(*n as f64)),
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::String(s) => {
                    // Trim whitespace and try parsing
                    let trimmed = s.trim();
                    match trimmed.parse::<f64>() {
                        Ok(f) => Ok(Value::Float(f)),
                        Err(_) => Err(RuntimeError {
                            message: format!("Cannot convert '{}' to float", s),
                        }),
                    }
                },
                _ => Err(RuntimeError {
                    message: format!("Cannot convert {} to float", args[0].type_name()),
                }),
            }
        },
        "to_string" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_string() takes exactly 1 argument".to_string(),
                });
            }
            Ok(Value::String(args[0].to_string()))
        },
        "to_bool" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "to_bool() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::Integer(n) => Ok(Value::Bool(*n != 0)),
                Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                Value::String(s) => {
                    match s.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
                        "false" | "0" | "no" | "off" | "" => Ok(Value::Bool(false)),
                        _ => Ok(Value::Bool(true)), // Non-empty strings are truthy by default
                    }
                }
                Value::Nil => Ok(Value::Bool(false)),
                _ => Ok(Value::Bool(args[0].is_truthy())),
            }
        },
        
        // Math functions
        "sign" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "sign() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => Ok(Value::Integer(if *n > 0 { 1 } else if *n < 0 { -1 } else { 0 })),
                Value::Float(f) => Ok(Value::Integer(if *f > 0.0 { 1 } else if *f < 0.0 { -1 } else { 0 })),
                _ => Err(RuntimeError {
                    message: "sign() requires a number".to_string(),
                }),
            }
        },
        "factorial" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "factorial() takes exactly 1 argument".to_string(),
                });
            }
            match &args[0] {
                Value::Integer(n) => {
                    if *n < 0 {
                        return Err(RuntimeError {
                            message: "factorial() requires a non-negative integer".to_string(),
                        });
                    }
                    let mut result = 1i64;
                    for i in 1..=*n {
                        result *= i;
                    }
                    Ok(Value::Integer(result))
                }
                _ => Err(RuntimeError {
                    message: "factorial() requires an integer".to_string(),
                }),
            }
        },
        "random_int" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "random_int() takes exactly 2 arguments (min, max)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Integer(min), Value::Integer(max)) => {
                    if min >= max {
                        return Err(RuntimeError {
                            message: "random_int() min must be less than max".to_string(),
                        });
                    }
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    use std::time::{SystemTime, UNIX_EPOCH};
                    
                    let mut hasher = DefaultHasher::new();
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
                    let hash = hasher.finish();
                    let range = max - min;
                    let result = min + ((hash as i64) % range);
                    Ok(Value::Integer(result))
                }
                _ => Err(RuntimeError {
                    message: "random_int() requires two integers".to_string(),
                }),
            }
        },
        
        // I/O functions
        "println" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "println() takes exactly 1 argument".to_string(),
                });
            }
            println!("{}", args[0]);
            Ok(Value::Nil)
        },
        
        // File system functions
        "file_exists" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "file_exists() takes exactly 1 argument (path)".to_string(),
                });
            }
            match &args[0] {
                Value::String(path) => Ok(Value::Bool(std::path::Path::new(path).exists())),
                _ => Err(RuntimeError {
                    message: "file_exists() requires a string path".to_string(),
                }),
            }
        },
        "is_file" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_file() takes exactly 1 argument (path)".to_string(),
                });
            }
            match &args[0] {
                Value::String(path) => Ok(Value::Bool(std::path::Path::new(path).is_file())),
                _ => Err(RuntimeError {
                    message: "is_file() requires a string path".to_string(),
                }),
            }
        },
        "is_dir" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "is_dir() takes exactly 1 argument (path)".to_string(),
                });
            }
            match &args[0] {
                Value::String(path) => Ok(Value::Bool(std::path::Path::new(path).is_dir())),
                _ => Err(RuntimeError {
                    message: "is_dir() requires a string path".to_string(),
                }),
            }
        },
        
        "any" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "any() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    for value in arr {
                        if value.is_truthy() {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                }
                _ => Err(RuntimeError {
                    message: "any() requires an array".to_string(),
                }),
            }
        },
        "all" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "all() takes exactly 1 argument (array)".to_string(),
                });
            }
            match &args[0] {
                Value::Array(arr) => {
                    for value in arr {
                        if !value.is_truthy() {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err(RuntimeError {
                    message: "all() requires an array".to_string(),
                }),
            }
        },
        "contains_str" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "contains_str() takes exactly 2 arguments (string, substring)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub))),
                _ => Err(RuntimeError {
                    message: "contains_str() requires two strings".to_string(),
                }),
            }
        },
        "pad_left" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "pad_left() takes exactly 3 arguments (string, length, padding)".to_string(),
                });
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Integer(len), Value::String(pad)) => {
                    let target_len = *len as usize;
                    if s.len() >= target_len {
                        Ok(Value::String(s.clone()))
                    } else {
                        let pad_count = target_len - s.len();
                        let padding = pad.repeat(pad_count);
                        Ok(Value::String(format!("{}{}", padding, s)))
                    }
                }
                _ => Err(RuntimeError {
                    message: "pad_left() requires a string, integer length, and padding string".to_string(),
                }),
            }
        },
        "pad_right" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "pad_right() takes exactly 3 arguments (string, length, padding)".to_string(),
                });
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Integer(len), Value::String(pad)) => {
                    let target_len = *len as usize;
                    if s.len() >= target_len {
                        Ok(Value::String(s.clone()))
                    } else {
                        let pad_count = target_len - s.len();
                        let padding = pad.repeat(pad_count);
                        Ok(Value::String(format!("{}{}", s, padding)))
                    }
                }
                _ => Err(RuntimeError {
                    message: "pad_right() requires a string, integer length, and padding string".to_string(),
                }),
            }
        },
        "reverse_str" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "reverse_str() takes exactly 1 argument (string)".to_string(),
                });
            }
            match &args[0] {
                Value::String(s) => {
                    let reversed: String = s.chars().rev().collect();
                    Ok(Value::String(reversed))
                }
                _ => Err(RuntimeError {
                    message: "reverse_str() requires a string".to_string(),
                }),
            }
        },
        "assert" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(RuntimeError {
                    message: "assert() takes 1 or 2 arguments (condition, optional message)".to_string(),
                });
            }
            let condition = &args[0];
            if !condition.is_truthy() {
                let message = if args.len() == 2 {
                    match &args[1] {
                        Value::String(msg) => msg.clone(),
                        _ => "Assertion failed".to_string(),
                    }
                } else {
                    "Assertion failed".to_string()
                };
                return Err(RuntimeError {
                    message,
                });
            }
            Ok(Value::Nil)
        },
        
        // Collection functions
        "collection" => {
            // Creates a new empty collection or from array/string arguments
            let mut set = std::collections::HashSet::new();
            for arg in args {
                match arg {
                    Value::Array(arr) => {
                        for item in arr {
                            set.insert(item.to_string());
                        }
                    }
                    _ => {
                        set.insert(arg.to_string());
                    }
                }
            }
            Ok(Value::Collection(set))
        },
        "add_to" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "add_to() takes exactly 2 arguments (collection, item)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(set) => {
                    let mut new_set = set.clone();
                    new_set.insert(args[1].to_string());
                    Ok(Value::Collection(new_set))
                }
                _ => Err(RuntimeError {
                    message: "add_to() requires a collection as first argument".to_string(),
                }),
            }
        },
        "remove_from" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "remove_from() takes exactly 2 arguments (collection, item)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(set) => {
                    let mut new_set = set.clone();
                    new_set.remove(&args[1].to_string());
                    Ok(Value::Collection(new_set))
                }
                _ => Err(RuntimeError {
                    message: "remove_from() requires a collection as first argument".to_string(),
                }),
            }
        },
        "has" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "has() takes exactly 2 arguments (collection, item)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(set) => {
                    Ok(Value::Bool(set.contains(&args[1].to_string())))
                }
                _ => Err(RuntimeError {
                    message: "has() requires a collection as first argument".to_string(),
                }),
            }
        },
        "union" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "union() takes exactly 2 arguments (collection1, collection2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Collection(set1), Value::Collection(set2)) => {
                    let union_set: std::collections::HashSet<String> = set1.union(set2).cloned().collect();
                    Ok(Value::Collection(union_set))
                }
                _ => Err(RuntimeError {
                    message: "union() requires two collections".to_string(),
                }),
            }
        },
        "intersect" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "intersect() takes exactly 2 arguments (collection1, collection2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Collection(set1), Value::Collection(set2)) => {
                    let intersection_set: std::collections::HashSet<String> = set1.intersection(set2).cloned().collect();
                    Ok(Value::Collection(intersection_set))
                }
                _ => Err(RuntimeError {
                    message: "intersect() requires two collections".to_string(),
                }),
            }
        },
        "difference" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "difference() takes exactly 2 arguments (collection1, collection2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Collection(set1), Value::Collection(set2)) => {
                    let difference_set: std::collections::HashSet<String> = set1.difference(set2).cloned().collect();
                    Ok(Value::Collection(difference_set))
                }
                _ => Err(RuntimeError {
                    message: "difference() requires two collections".to_string(),
                }),
            }
        },
        "size" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "size() takes exactly 1 argument (collection)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(set) => {
                    Ok(Value::Integer(set.len() as i64))
                }
                Value::Array(arr) => {
                    Ok(Value::Integer(arr.len() as i64))
                }
                Value::String(s) => {
                    Ok(Value::Integer(s.chars().count() as i64))
                }
                Value::Dictionary(dict) => {
                    Ok(Value::Integer(dict.len() as i64))
                }
                _ => Err(RuntimeError {
                    message: "size() requires a collection, array, string, or dictionary".to_string(),
                }),
            }
        },
        "is_subset" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "is_subset() takes exactly 2 arguments (collection1, collection2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Collection(set1), Value::Collection(set2)) => {
                    Ok(Value::Bool(set1.is_subset(set2)))
                }
                _ => Err(RuntimeError {
                    message: "is_subset() requires two collections".to_string(),
                }),
            }
        },
        "is_superset" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "is_superset() takes exactly 2 arguments (collection1, collection2)".to_string(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::Collection(set1), Value::Collection(set2)) => {
                    Ok(Value::Bool(set1.is_superset(set2)))
                }
                _ => Err(RuntimeError {
                    message: "is_superset() requires two collections".to_string(),
                }),
            }
        },
        "clear_collection" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "clear_collection() takes exactly 1 argument (collection)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(_) => {
                    Ok(Value::Collection(std::collections::HashSet::new()))
                }
                _ => Err(RuntimeError {
                    message: "clear_collection() requires a collection".to_string(),
                }),
            }
        },
        "to_array" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(RuntimeError {
                    message: "to_array() takes 1 or 2 arguments (collection/string, optional delimiter)".to_string(),
                });
            }
            match &args[0] {
                Value::Collection(set) => {
                    if args.len() == 2 {
                        return Err(RuntimeError {
                            message: "to_array() delimiter parameter only works with strings".to_string(),
                        });
                    }
                    let mut items: Vec<String> = set.iter().cloned().collect();
                    items.sort(); // Sort for consistent output
                    let values: Vec<Value> = items.into_iter().map(Value::String).collect();
                    Ok(Value::Array(values))
                }
                Value::UniqueArray(uarr) => {
                    // Convert unique array to regular array
                    if args.len() == 2 {
                        return Err(RuntimeError {
                            message: "to_array() delimiter parameter only works with strings".to_string(),
                        });
                    }
                    Ok(Value::Array(uarr.clone()))
                }
                Value::String(s) => {
                    if args.len() == 2 {
                        // to_array(string, delimiter) - split by delimiter
                        match &args[1] {
                            Value::String(delimiter) => {
                                if delimiter.is_empty() {
                                    // Empty delimiter means split into characters (same as 1-arg version)
                                    let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                                    Ok(Value::Array(chars))
                                } else {
                                    // Split by delimiter
                                    let parts: Vec<Value> = s.split(delimiter).map(|part| Value::String(part.to_string())).collect();
                                    Ok(Value::Array(parts))
                                }
                            }
                            _ => Err(RuntimeError {
                                message: "to_array() delimiter must be a string".to_string(),
                            }),
                        }
                    } else {
                        // to_array(string) - split into characters
                        let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                        Ok(Value::Array(chars))
                    }
                }
                _ => Err(RuntimeError {
                    message: "to_array() requires a collection, unique array, or string".to_string(),
                }),
            }
        },
        
        "input" => {
            let prompt = if args.len() > 0 {
                match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            };
            
            if !prompt.is_empty() {
                print!("{}", prompt);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            
            let mut line = String::new();
            match std::io::stdin().read_line(&mut line) {
                Ok(_) => Ok(Value::String(line.trim_end().to_string())),
                Err(_) => Ok(Value::String("".to_string())),
            }
        },
        
        "exec" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "exec() takes exactly 1 argument (command)".to_string(),
                });
            }
            match &args[0] {
                Value::String(cmd) => {
                    use std::process::Command;
                    let output = if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args(["/C", cmd])
                            .output()
                    } else {
                        Command::new("sh")
                            .args(["-c", cmd])
                            .output()
                    };
                    
                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            Ok(Value::String(stdout.trim_end().to_string()))
                        }
                        Err(e) => Err(RuntimeError {
                            message: format!("Failed to execute command: {}", e),
                        }),
                    }
                }
                _ => Err(RuntimeError {
                    message: "exec() requires a string command".to_string(),
                }),
            }
        },
        
        "exit" => {
            let code = if args.len() > 0 {
                match &args[0] {
                    Value::Integer(n) => *n as i32,
                    Value::Float(f) => *f as i32,
                    _ => 0,
                }
            } else {
                0
            };
            std::process::exit(code);
        },

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
        Value::StructInstance { struct_name, fields } => {
            // Convert struct to JSON object
            let mut json_obj = serde_json::Map::new();
            json_obj.insert("_type".to_string(), serde_json::Value::String(struct_name.clone()));
            for (key, value) in fields {
                json_obj.insert(key.clone(), ject_value_to_json(value)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        }
        Value::StructDefinition { name, fields } => {
            // Convert struct definition to JSON
            let mut json_obj = serde_json::Map::new();
            json_obj.insert("_type".to_string(), serde_json::Value::String("struct_definition".to_string()));
            json_obj.insert("name".to_string(), serde_json::Value::String(name.clone()));
            let fields_array: Vec<serde_json::Value> = fields.iter().map(|f| serde_json::Value::String(f.clone())).collect();
            json_obj.insert("fields".to_string(), serde_json::Value::Array(fields_array));
            Ok(serde_json::Value::Object(json_obj))
        }
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
        Value::UniqueArray(arr) => {
            // UniqueArray converts to regular JSON array
            let json_array: Result<Vec<serde_json::Value>, RuntimeError> =
                arr.iter().map(ject_value_to_json).collect();
            match json_array {
                Ok(json_arr) => Ok(serde_json::Value::Array(json_arr)),
                Err(e) => Err(e),
            }
        }
        Value::Dictionary(dict) => {
            let mut json_obj = serde_json::Map::new();
            for (key, value) in dict {
                let json_value = ject_value_to_json(value)?;
                json_obj.insert(key.clone(), json_value);
            }
            Ok(serde_json::Value::Object(json_obj))
        }
        Value::Collection(set) => {
            let json_array: Vec<serde_json::Value> = set.iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect();
            Ok(serde_json::Value::Array(json_array))
        }
        Value::Function { .. } | Value::ModuleFunction { .. } | Value::Lambda { .. } | Value::BuiltinFunction(_) | Value::ModuleObject(_) => {
            Err(RuntimeError {
                message: "Cannot convert function to JSON".to_string(),
            })
        }
        Value::Error(msg) => {
            let mut json_obj = serde_json::Map::new();
            json_obj.insert("_type".to_string(), serde_json::Value::String("error".to_string()));
            json_obj.insert("message".to_string(), serde_json::Value::String(msg.clone()));
            Ok(serde_json::Value::Object(json_obj))
        }
        Value::NdArray(arr) => {
            // Convert ndarray to JSON array
            let data = match arr {
                crate::numpy::NdArray::F64(a) => a.iter().map(|&x| {
                    serde_json::Number::from_f64(x).unwrap_or(serde_json::Number::from(0))
                }).collect::<Vec<_>>(),
                crate::numpy::NdArray::I64(a) => a.iter().map(|&x| {
                    serde_json::Number::from(x)
                }).collect::<Vec<_>>(),
                crate::numpy::NdArray::Bool(a) => a.iter().map(|&x| {
                    serde_json::Number::from(if x { 1 } else { 0 })
                }).collect::<Vec<_>>(),
            };
            Ok(serde_json::Value::Array(data.into_iter().map(serde_json::Value::Number).collect()))
        }
    }
}
