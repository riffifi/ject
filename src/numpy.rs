//! NumPy-like Array Library for Ject
//! Provides high-performance numerical computing capabilities
//! Built on top of the ndarray crate

use crate::value::Value;
use crate::interpreter::RuntimeError;
use std::collections::HashMap;
use ndarray::{Array1, Array2, ArrayD, Dimension};

/// N-dimensional array wrapper
#[derive(Clone)]
pub enum NdArray {
    F64(ArrayD<f64>),
    I64(ArrayD<i64>),
    Bool(ArrayD<bool>),
}

impl std::fmt::Debug for NdArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NdArray::F64(a) => write!(f, "NdArray(F64, shape={:?})", a.shape()),
            NdArray::I64(a) => write!(f, "NdArray(I64, shape={:?})", a.shape()),
            NdArray::Bool(a) => write!(f, "NdArray(Bool, shape={:?})", a.shape()),
        }
    }
}

impl PartialEq for NdArray {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NdArray::F64(a), NdArray::F64(b)) => {
                if a.shape() != b.shape() { return false; }
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-10)
            }
            (NdArray::I64(a), NdArray::I64(b)) => {
                if a.shape() != b.shape() { return false; }
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            (NdArray::Bool(a), NdArray::Bool(b)) => {
                if a.shape() != b.shape() { return false; }
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            _ => false,
        }
    }
}

impl NdArray {
    fn len(&self) -> usize {
        match self {
            NdArray::F64(a) => a.len(),
            NdArray::I64(a) => a.len(),
            NdArray::Bool(a) => a.len(),
        }
    }

    fn shape(&self) -> Vec<usize> {
        match self {
            NdArray::F64(a) => a.shape().to_vec(),
            NdArray::I64(a) => a.shape().to_vec(),
            NdArray::Bool(a) => a.shape().to_vec(),
        }
    }

    fn ndim(&self) -> usize {
        match self {
            NdArray::F64(a) => a.ndim(),
            NdArray::I64(a) => a.ndim(),
            NdArray::Bool(a) => a.ndim(),
        }
    }

    fn to_f64_vec(&self) -> Result<Vec<f64>, RuntimeError> {
        match self {
            NdArray::F64(a) => Ok(a.iter().copied().collect()),
            NdArray::I64(a) => Ok(a.iter().map(|&x| x as f64).collect()),
            NdArray::Bool(a) => Ok(a.iter().map(|&x| if x { 1.0 } else { 0.0 }).collect()),
        }
    }
}

/// Create numpy module
pub fn create_numpy_module() -> HashMap<String, Value> {
    let mut module = HashMap::new();

    // Array creation
    module.insert("array".to_string(), Value::BuiltinFunction("np_array".to_string()));
    module.insert("zeros".to_string(), Value::BuiltinFunction("np_zeros".to_string()));
    module.insert("ones".to_string(), Value::BuiltinFunction("np_ones".to_string()));
    module.insert("arange".to_string(), Value::BuiltinFunction("np_arange".to_string()));
    module.insert("linspace".to_string(), Value::BuiltinFunction("np_linspace".to_string()));
    module.insert("eye".to_string(), Value::BuiltinFunction("np_eye".to_string()));
    module.insert("identity".to_string(), Value::BuiltinFunction("np_identity".to_string()));
    module.insert("full".to_string(), Value::BuiltinFunction("np_full".to_string()));
    module.insert("empty".to_string(), Value::BuiltinFunction("np_empty".to_string()));

    // Array properties
    module.insert("shape".to_string(), Value::BuiltinFunction("np_shape".to_string()));
    module.insert("ndim".to_string(), Value::BuiltinFunction("np_ndim".to_string()));
    module.insert("size".to_string(), Value::BuiltinFunction("np_size".to_string()));
    module.insert("dtype".to_string(), Value::BuiltinFunction("np_dtype".to_string()));

    // Array manipulation
    module.insert("reshape".to_string(), Value::BuiltinFunction("np_reshape".to_string()));
    module.insert("flatten".to_string(), Value::BuiltinFunction("np_flatten".to_string()));
    module.insert("ravel".to_string(), Value::BuiltinFunction("np_ravel".to_string()));
    module.insert("transpose".to_string(), Value::BuiltinFunction("np_transpose".to_string()));
    module.insert("swapaxes".to_string(), Value::BuiltinFunction("np_swapaxes".to_string()));
    module.insert("squeeze".to_string(), Value::BuiltinFunction("np_squeeze".to_string()));
    module.insert("expand_dims".to_string(), Value::BuiltinFunction("np_expand_dims".to_string()));
    module.insert("concatenate".to_string(), Value::BuiltinFunction("np_concatenate".to_string()));
    module.insert("stack".to_string(), Value::BuiltinFunction("np_stack".to_string()));
    module.insert("vstack".to_string(), Value::BuiltinFunction("np_vstack".to_string()));
    module.insert("hstack".to_string(), Value::BuiltinFunction("np_hstack".to_string()));
    module.insert("split".to_string(), Value::BuiltinFunction("np_split".to_string()));
    module.insert("hsplit".to_string(), Value::BuiltinFunction("np_hsplit".to_string()));
    module.insert("vsplit".to_string(), Value::BuiltinFunction("np_vsplit".to_string()));

    // Mathematical operations
    module.insert("add".to_string(), Value::BuiltinFunction("np_add".to_string()));
    module.insert("subtract".to_string(), Value::BuiltinFunction("np_subtract".to_string()));
    module.insert("multiply".to_string(), Value::BuiltinFunction("np_multiply".to_string()));
    module.insert("divide".to_string(), Value::BuiltinFunction("np_divide".to_string()));
    module.insert("power".to_string(), Value::BuiltinFunction("np_power".to_string()));
    module.insert("sqrt".to_string(), Value::BuiltinFunction("np_sqrt".to_string()));
    module.insert("exp".to_string(), Value::BuiltinFunction("np_exp".to_string()));
    module.insert("log".to_string(), Value::BuiltinFunction("np_log".to_string()));
    module.insert("log10".to_string(), Value::BuiltinFunction("np_log10".to_string()));
    module.insert("abs".to_string(), Value::BuiltinFunction("np_abs".to_string()));
    module.insert("negative".to_string(), Value::BuiltinFunction("np_negative".to_string()));
    module.insert("ceil".to_string(), Value::BuiltinFunction("np_ceil".to_string()));
    module.insert("floor".to_string(), Value::BuiltinFunction("np_floor".to_string()));
    module.insert("round".to_string(), Value::BuiltinFunction("np_round".to_string()));
    module.insert("clip".to_string(), Value::BuiltinFunction("np_clip".to_string()));
    module.insert("nan_to_num".to_string(), Value::BuiltinFunction("np_nan_to_num".to_string()));

    // Trigonometric functions
    module.insert("sin".to_string(), Value::BuiltinFunction("np_sin".to_string()));
    module.insert("cos".to_string(), Value::BuiltinFunction("np_cos".to_string()));
    module.insert("tan".to_string(), Value::BuiltinFunction("np_tan".to_string()));
    module.insert("arcsin".to_string(), Value::BuiltinFunction("np_arcsin".to_string()));
    module.insert("arccos".to_string(), Value::BuiltinFunction("np_arccos".to_string()));
    module.insert("arctan".to_string(), Value::BuiltinFunction("np_arctan".to_string()));
    module.insert("arctan2".to_string(), Value::BuiltinFunction("np_arctan2".to_string()));
    module.insert("hypot".to_string(), Value::BuiltinFunction("np_hypot".to_string()));
    module.insert("degrees".to_string(), Value::BuiltinFunction("np_degrees".to_string()));
    module.insert("radians".to_string(), Value::BuiltinFunction("np_radians".to_string()));

    // Hyperbolic functions
    module.insert("sinh".to_string(), Value::BuiltinFunction("np_sinh".to_string()));
    module.insert("cosh".to_string(), Value::BuiltinFunction("np_cosh".to_string()));
    module.insert("tanh".to_string(), Value::BuiltinFunction("np_tanh".to_string()));
    module.insert("arcsinh".to_string(), Value::BuiltinFunction("np_arcsinh".to_string()));
    module.insert("arccosh".to_string(), Value::BuiltinFunction("np_arccosh".to_string()));
    module.insert("arctanh".to_string(), Value::BuiltinFunction("np_arctanh".to_string()));

    // Reduction operations
    module.insert("sum".to_string(), Value::BuiltinFunction("np_sum".to_string()));
    module.insert("prod".to_string(), Value::BuiltinFunction("np_prod".to_string()));
    module.insert("mean".to_string(), Value::BuiltinFunction("np_mean".to_string()));
    module.insert("std".to_string(), Value::BuiltinFunction("np_std".to_string()));
    module.insert("var".to_string(), Value::BuiltinFunction("np_var".to_string()));
    module.insert("min".to_string(), Value::BuiltinFunction("np_min".to_string()));
    module.insert("max".to_string(), Value::BuiltinFunction("np_max".to_string()));
    module.insert("argmin".to_string(), Value::BuiltinFunction("np_argmin".to_string()));
    module.insert("argmax".to_string(), Value::BuiltinFunction("np_argmax".to_string()));
    module.insert("cumsum".to_string(), Value::BuiltinFunction("np_cumsum".to_string()));
    module.insert("cumprod".to_string(), Value::BuiltinFunction("np_cumprod".to_string()));
    module.insert("any".to_string(), Value::BuiltinFunction("np_any".to_string()));
    module.insert("all".to_string(), Value::BuiltinFunction("np_all".to_string()));

    // Linear algebra
    module.insert("dot".to_string(), Value::BuiltinFunction("np_dot".to_string()));
    module.insert("inner".to_string(), Value::BuiltinFunction("np_inner".to_string()));
    module.insert("outer".to_string(), Value::BuiltinFunction("np_outer".to_string()));
    module.insert("matmul".to_string(), Value::BuiltinFunction("np_matmul".to_string()));
    module.insert("tensordot".to_string(), Value::BuiltinFunction("np_tensordot".to_string()));
    module.insert("cross".to_string(), Value::BuiltinFunction("np_cross".to_string()));
    module.insert("trace".to_string(), Value::BuiltinFunction("np_trace".to_string()));
    module.insert("diag".to_string(), Value::BuiltinFunction("np_diag".to_string()));
    module.insert("tri".to_string(), Value::BuiltinFunction("np_tri".to_string()));
    module.insert("tril".to_string(), Value::BuiltinFunction("np_tril".to_string()));
    module.insert("triu".to_string(), Value::BuiltinFunction("np_triu".to_string()));

    // Sorting and searching
    module.insert("sort".to_string(), Value::BuiltinFunction("np_sort".to_string()));
    module.insert("argsort".to_string(), Value::BuiltinFunction("np_argsort".to_string()));
    module.insert("lexsort".to_string(), Value::BuiltinFunction("np_lexsort".to_string()));
    module.insert("searchsorted".to_string(), Value::BuiltinFunction("np_searchsorted".to_string()));
    module.insert("nonzero".to_string(), Value::BuiltinFunction("np_nonzero".to_string()));
    module.insert("where".to_string(), Value::BuiltinFunction("np_where".to_string()));

    // Set operations
    module.insert("unique".to_string(), Value::BuiltinFunction("np_unique".to_string()));
    module.insert("in1d".to_string(), Value::BuiltinFunction("np_in1d".to_string()));
    module.insert("intersect1d".to_string(), Value::BuiltinFunction("np_intersect1d".to_string()));
    module.insert("union1d".to_string(), Value::BuiltinFunction("np_union1d".to_string()));
    module.insert("setdiff1d".to_string(), Value::BuiltinFunction("np_setdiff1d".to_string()));
    module.insert("setxor1d".to_string(), Value::BuiltinFunction("np_setxor1d".to_string()));

    // Logical operations
    module.insert("logical_and".to_string(), Value::BuiltinFunction("np_logical_and".to_string()));
    module.insert("logical_or".to_string(), Value::BuiltinFunction("np_logical_or".to_string()));
    module.insert("logical_not".to_string(), Value::BuiltinFunction("np_logical_not".to_string()));
    module.insert("logical_xor".to_string(), Value::BuiltinFunction("np_logical_xor".to_string()));
    module.insert("greater".to_string(), Value::BuiltinFunction("np_greater".to_string()));
    module.insert("greater_equal".to_string(), Value::BuiltinFunction("np_greater_equal".to_string()));
    module.insert("less".to_string(), Value::BuiltinFunction("np_less".to_string()));
    module.insert("less_equal".to_string(), Value::BuiltinFunction("np_less_equal".to_string()));
    module.insert("equal".to_string(), Value::BuiltinFunction("np_equal".to_string()));
    module.insert("not_equal".to_string(), Value::BuiltinFunction("np_not_equal".to_string()));

    // Random module (simplified)
    module.insert("random".to_string(), Value::BuiltinFunction("np_random".to_string()));
    module.insert("rand".to_string(), Value::BuiltinFunction("np_rand".to_string()));
    module.insert("randn".to_string(), Value::BuiltinFunction("np_randn".to_string()));
    module.insert("randint".to_string(), Value::BuiltinFunction("np_randint".to_string()));
    module.insert("choice".to_string(), Value::BuiltinFunction("np_choice".to_string()));
    module.insert("shuffle".to_string(), Value::BuiltinFunction("np_shuffle".to_string()));
    module.insert("permutation".to_string(), Value::BuiltinFunction("np_permutation".to_string()));
    module.insert("seed".to_string(), Value::BuiltinFunction("np_seed".to_string()));

    // Constants
    module.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
    module.insert("E".to_string(), Value::Float(std::f64::consts::E));
    module.insert("inf".to_string(), Value::Float(f64::INFINITY));
    module.insert("nan".to_string(), Value::Float(f64::NAN));

    module
}

/// Call numpy builtin function
pub fn call_numpy_function(name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match name {
        // Array creation
        "np_array" => np_array(args),
        "np_zeros" => np_zeros(args),
        "np_ones" => np_ones(args),
        "np_arange" => np_arange(args),
        "np_linspace" => np_linspace(args),
        "np_eye" => np_eye(args),
        "np_identity" => np_identity(args),
        "np_full" => np_full(args),
        "np_empty" => np_empty(args),

        // Array properties
        "np_shape" => np_shape(args),
        "np_ndim" => np_ndim(args),
        "np_size" => np_size(args),
        "np_dtype" => np_dtype(args),

        // Array manipulation
        "np_reshape" => np_reshape(args),
        "np_flatten" => np_flatten(args),
        "np_ravel" => np_ravel(args),
        "np_transpose" => np_transpose(args),
        "np_swapaxes" => np_swapaxes(args),
        "np_squeeze" => np_squeeze(args),
        "np_expand_dims" => np_expand_dims(args),
        "np_concatenate" => np_concatenate(args),
        "np_stack" => np_stack(args),
        "np_vstack" => np_vstack(args),
        "np_hstack" => np_hstack(args),
        "np_split" => np_split(args),

        // Mathematical operations
        "np_add" => np_add(args),
        "np_subtract" => np_subtract(args),
        "np_multiply" => np_multiply(args),
        "np_divide" => np_divide(args),
        "np_power" => np_power(args),
        "np_sqrt" => np_sqrt(args),
        "np_exp" => np_exp(args),
        "np_log" => np_log(args),
        "np_log10" => np_log10(args),
        "np_abs" => np_abs(args),
        "np_negative" => np_negative(args),
        "np_ceil" => np_ceil(args),
        "np_floor" => np_floor(args),
        "np_round" => np_round(args),
        "np_clip" => np_clip(args),

        // Trigonometric
        "np_sin" => np_sin(args),
        "np_cos" => np_cos(args),
        "np_tan" => np_tan(args),
        "np_arcsin" => np_arcsin(args),
        "np_arccos" => np_arccos(args),
        "np_arctan" => np_arctan(args),
        "np_arctan2" => np_arctan2(args),
        "np_degrees" => np_degrees(args),
        "np_radians" => np_radians(args),

        // Hyperbolic
        "np_sinh" => np_sinh(args),
        "np_cosh" => np_cosh(args),
        "np_tanh" => np_tanh(args),

        // Reduction
        "np_sum" => np_sum(args),
        "np_mean" => np_mean(args),
        "np_std" => np_std(args),
        "np_var" => np_var(args),
        "np_min" => np_min(args),
        "np_max" => np_max(args),
        "np_argmin" => np_argmin(args),
        "np_argmax" => np_argmax(args),
        "np_cumsum" => np_cumsum(args),
        "np_any" => np_any(args),
        "np_all" => np_all(args),

        // Linear algebra
        "np_dot" => np_dot(args),
        "np_inner" => np_inner(args),
        "np_outer" => np_outer(args),
        "np_matmul" => np_matmul(args),
        "np_trace" => np_trace(args),
        "np_diag" => np_diag(args),

        // Sorting
        "np_sort" => np_sort(args),
        "np_argsort" => np_argsort(args),
        "np_where" => np_where(args),

        // Set operations
        "np_unique" => np_unique(args),

        // Logical
        "np_logical_and" => np_logical_and(args),
        "np_logical_or" => np_logical_or(args),
        "np_logical_not" => np_logical_not(args),
        "np_greater" => np_greater(args),
        "np_less" => np_less(args),
        "np_equal" => np_equal(args),
        "np_not_equal" => np_not_equal(args),

        // Random
        "np_random" => np_random(args),
        "np_rand" => np_rand(args),
        "np_randint" => np_randint(args),

        _ => Err(RuntimeError {
            message: format!("Unknown numpy function: {}", name),
        }),
    }
}

// ============================================================================
// Array Creation Functions
// ============================================================================

fn np_array(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "array() requires at least one argument".to_string(),
        });
    }

    match &args[0] {
        Value::Array(arr) => {
            let data: Vec<f64> = arr.iter()
                .map(|v| match v {
                    Value::Integer(i) => *i as f64,
                    Value::Float(f) => *f,
                    Value::Bool(b) => if *b { 1.0 } else { 0.0 },
                    _ => 0.0,
                })
                .collect();
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![data.len()], data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "array() requires an array argument".to_string(),
        }),
    }
}

fn np_zeros(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "zeros() requires a shape argument".to_string(),
        });
    }

    match &args[0] {
        Value::Array(shape_arr) => {
            let shape: Vec<usize> = shape_arr.iter()
                .filter_map(|v| match v {
                    Value::Integer(i) => Some(*i as usize),
                    _ => None,
                })
                .collect();
            
            if shape.is_empty() {
                return Err(RuntimeError {
                    message: "zeros() requires a valid shape".to_string(),
                });
            }

            let size = shape.iter().product();
            let data = vec![0.0f64; size];
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        Value::Integer(n) => {
            let shape = vec![*n as usize];
            let data = vec![0.0f64; *n as usize];
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "zeros() requires an integer or array shape argument".to_string(),
        }),
    }
}

fn np_ones(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "ones() requires a shape argument".to_string(),
        });
    }

    match &args[0] {
        Value::Array(shape_arr) => {
            let shape: Vec<usize> = shape_arr.iter()
                .filter_map(|v| match v {
                    Value::Integer(i) => Some(*i as usize),
                    _ => None,
                })
                .collect();
            
            if shape.is_empty() {
                return Err(RuntimeError {
                    message: "ones() requires a valid shape".to_string(),
                });
            }

            let size = shape.iter().product();
            let data = vec![1.0f64; size];
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        Value::Integer(n) => {
            let shape = vec![*n as usize];
            let data = vec![1.0f64; *n as usize];
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "ones() requires an integer or array shape argument".to_string(),
        }),
    }
}

fn np_arange(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (start, stop, step) = match args.len() {
        1 => (0.0, value_to_f64(&args[0])?, 1.0),
        2 => (value_to_f64(&args[0])?, value_to_f64(&args[1])?, 1.0),
        3 => (value_to_f64(&args[0])?, value_to_f64(&args[1])?, value_to_f64(&args[2])?),
        _ => return Err(RuntimeError {
            message: "arange() takes 1, 2, or 3 arguments (start, stop, step)".to_string(),
        }),
    };

    let mut data = Vec::new();
    let mut current = start;
    if step > 0.0 {
        while current < stop {
            data.push(current);
            current += step;
        }
    } else {
        while current > stop {
            data.push(current);
            current += step;
        }
    }

    let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![data.len()], data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_linspace(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "linspace() requires at least start and stop arguments".to_string(),
        });
    }

    let start = value_to_f64(&args[0])?;
    let stop = value_to_f64(&args[1])?;
    let num = if args.len() >= 3 {
        value_to_i64(&args[2])? as usize
    } else {
        50
    };

    let mut data = Vec::with_capacity(num);
    for i in 0..num {
        let t = if num > 1 { i as f64 / (num - 1) as f64 } else { 0.0 };
        data.push(start + t * (stop - start));
    }

    let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![num], data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_eye(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "eye() requires at least N argument".to_string(),
        });
    }

    let n = value_to_i64(&args[0])? as usize;
    let m = if args.len() >= 2 {
        value_to_i64(&args[1])? as usize
    } else {
        n
    };

    let mut data = vec![0.0f64; n * m];
    for i in 0..n.min(m) {
        data[i * m + i] = 1.0;
    }

    let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n, m], data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_identity(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "identity() requires N argument".to_string(),
        });
    }

    let n = value_to_i64(&args[0])? as usize;
    let mut data = vec![0.0f64; n * n];
    for i in 0..n {
        data[i * n + i] = 1.0;
    }

    let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n, n], data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_full(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "full() requires shape and fill_value arguments".to_string(),
        });
    }

    let shape = match &args[0] {
        Value::Array(shape_arr) => shape_arr.iter()
            .filter_map(|v| match v {
                Value::Integer(i) => Some(*i as usize),
                _ => None,
            })
            .collect::<Vec<usize>>(),
        Value::Integer(n) => vec![*n as usize],
        _ => return Err(RuntimeError {
            message: "full() requires an array or integer shape".to_string(),
        }),
    };

    let fill_value = value_to_f64(&args[1])?;
    let size = shape.iter().product();
    let data = vec![fill_value; size];

    let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_empty(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Empty array (initialized to zeros for safety)
    np_zeros(args)
}

// ============================================================================
// Array Property Functions
// ============================================================================

fn np_shape(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "shape() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let shape: Vec<Value> = arr.shape().iter().map(|&d| Value::Integer(d as i64)).collect();
            Ok(Value::Array(shape))
        }
        Value::Array(arr) => {
            Ok(Value::Array(vec![Value::Integer(arr.len() as i64)]))
        }
        _ => Err(RuntimeError {
            message: "shape() requires an array argument".to_string(),
        }),
    }
}

fn np_ndim(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "ndim() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => Ok(Value::Integer(arr.ndim() as i64)),
        Value::Array(_) => Ok(Value::Integer(1)),
        _ => Err(RuntimeError {
            message: "ndim() requires an array argument".to_string(),
        }),
    }
}

fn np_size(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "size() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        _ => Err(RuntimeError {
            message: "size() requires an array argument".to_string(),
        }),
    }
}

fn np_dtype(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "dtype() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let dtype = match arr {
                NdArray::F64(_) => "float64",
                NdArray::I64(_) => "int64",
                NdArray::Bool(_) => "bool",
            };
            Ok(Value::String(dtype.to_string()))
        }
        _ => Ok(Value::String("unknown".to_string())),
    }
}

// ============================================================================
// Array Manipulation Functions
// ============================================================================

fn np_reshape(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "reshape() requires array and shape arguments".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let new_shape = match &args[1] {
                Value::Array(shape_arr) => shape_arr.iter()
                    .filter_map(|v| match v {
                        Value::Integer(i) => Some(*i as usize),
                        _ => None,
                    })
                    .collect::<Vec<usize>>(),
                Value::Integer(n) => vec![*n as usize],
                _ => return Err(RuntimeError {
                    message: "reshape() requires an array or integer shape".to_string(),
                }),
            };

            let data = arr.to_f64_vec()?;
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(new_shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "reshape() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_flatten(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "flatten() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![data.len()], data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "flatten() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_ravel(args: Vec<Value>) -> Result<Value, RuntimeError> {
    np_flatten(args)
}

fn np_transpose(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "transpose() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let shape = arr.shape();
            
            if shape.len() != 2 {
                return Err(RuntimeError {
                    message: "transpose() currently supports 2D arrays only".to_string(),
                });
            }

            let n = shape[0];
            let m = shape[1];
            let mut transposed = vec![0.0f64; n * m];
            
            for i in 0..n {
                for j in 0..m {
                    transposed[j * n + i] = data[i * m + j];
                }
            }

            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![m, n], transposed).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "transpose() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_swapaxes(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Simplified: just transpose for 2D
    if args.len() < 3 {
        return Err(RuntimeError {
            message: "swapaxes() requires array, axis1, axis2 arguments".to_string(),
        });
    }
    np_transpose(vec![args[0].clone()])
}

fn np_squeeze(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "squeeze() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let new_shape = arr.shape().iter()
                .filter(|&&d| d > 1)
                .copied()
                .collect::<Vec<usize>>();
            
            let shape = if new_shape.is_empty() { vec![1] } else { new_shape };
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "squeeze() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_expand_dims(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "expand_dims() requires array and axis arguments".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let mut new_shape = arr.shape().clone();
            let axis = value_to_i64(&args[1])? as usize;
            
            if axis > new_shape.len() {
                return Err(RuntimeError {
                    message: "expand_dims() axis out of bounds".to_string(),
                });
            }
            
            new_shape.insert(axis, 1);
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(new_shape, data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "expand_dims() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_concatenate(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "concatenate() requires a sequence of arrays".to_string(),
        });
    }

    match &args[0] {
        Value::Array(arrays) => {
            let mut all_data = Vec::new();
            for arr in arrays {
                match arr {
                    Value::NdArray(a) => {
                        all_data.extend(a.to_f64_vec()?);
                    }
                    Value::Array(a) => {
                        for v in a {
                            all_data.push(value_to_f64(v)?);
                        }
                    }
                    _ => {}
                }
            }
            
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![all_data.len()], all_data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "concatenate() requires an array of arrays".to_string(),
        }),
    }
}

fn np_stack(args: Vec<Value>) -> Result<Value, RuntimeError> {
    np_concatenate(args)
}

fn np_vstack(args: Vec<Value>) -> Result<Value, RuntimeError> {
    np_concatenate(args)
}

fn np_hstack(args: Vec<Value>) -> Result<Value, RuntimeError> {
    np_concatenate(args)
}

fn np_split(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Simplified implementation
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "split() requires array and indices arguments".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let _data = arr.to_f64_vec()?;
            let result = vec![Value::NdArray(arr.clone())];
            Ok(Value::Array(result))
        }
        _ => Err(RuntimeError {
            message: "split() requires an ndarray argument".to_string(),
        }),
    }
}

// ============================================================================
// Mathematical Functions
// ============================================================================

fn np_add(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "add() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| a + b)
}

fn np_subtract(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "subtract() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| a - b)
}

fn np_multiply(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "multiply() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| a * b)
}

fn np_divide(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "divide() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| a / b)
}

fn np_power(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "power() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| a.powf(b))
}

fn np_sqrt(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "sqrt() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.sqrt())
}

fn np_exp(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "exp() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.exp())
}

fn np_log(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "log() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.ln())
}

fn np_log10(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "log10() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.log10())
}

fn np_abs(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "abs() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.abs())
}

fn np_negative(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "negative() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| -x)
}

fn np_ceil(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "ceil() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.ceil())
}

fn np_floor(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "floor() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.floor())
}

fn np_round(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "round() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.round())
}

fn np_clip(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError {
            message: "clip() requires array, min, and max arguments".to_string(),
        });
    }
    let min_val = value_to_f64(&args[1])?;
    let max_val = value_to_f64(&args[2])?;
    unary_op(&args[0], |x| x.clamp(min_val, max_val))
}

// Trigonometric
fn np_sin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "sin() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.sin())
}

fn np_cos(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "cos() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.cos())
}

fn np_tan(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "tan() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.tan())
}

fn np_arcsin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "arcsin() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.asin())
}

fn np_arccos(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "arccos() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.acos())
}

fn np_arctan(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "arctan() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.atan())
}

fn np_arctan2(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "arctan2() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |y, x| y.atan2(x))
}

fn np_degrees(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "degrees() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.to_degrees())
}

fn np_radians(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "radians() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.to_radians())
}

// Hyperbolic
fn np_sinh(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "sinh() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.sinh())
}

fn np_cosh(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "cosh() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.cosh())
}

fn np_tanh(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "tanh() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| x.tanh())
}

// ============================================================================
// Reduction Functions
// ============================================================================

fn np_sum(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "sum() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let sum: f64 = data.iter().sum();
            Ok(Value::Float(sum))
        }
        Value::Array(arr) => {
            let sum: f64 = arr.iter().map(|v| value_to_f64(v).unwrap_or(0.0)).sum();
            Ok(Value::Float(sum))
        }
        _ => Err(RuntimeError {
            message: "sum() requires an array argument".to_string(),
        }),
    }
}

fn np_mean(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "mean() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let sum: f64 = data.iter().sum();
            Ok(Value::Float(sum / data.len() as f64))
        }
        Value::Array(arr) => {
            let sum: f64 = arr.iter().map(|v| value_to_f64(v).unwrap_or(0.0)).sum();
            Ok(Value::Float(sum / arr.len() as f64))
        }
        _ => Err(RuntimeError {
            message: "mean() requires an array argument".to_string(),
        }),
    }
}

fn np_std(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "std() requires an array argument".to_string(),
        });
    }

    let mean_val = np_mean(vec![args[0].clone()])?;
    let mean = match mean_val {
        Value::Float(f) => f,
        _ => return Err(RuntimeError { message: "Internal error".to_string() }),
    };

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
            Ok(Value::Float(variance.sqrt()))
        }
        Value::Array(arr) => {
            let data: Vec<f64> = arr.iter().map(|v| value_to_f64(v).unwrap_or(0.0)).collect();
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
            Ok(Value::Float(variance.sqrt()))
        }
        _ => Err(RuntimeError {
            message: "std() requires an array argument".to_string(),
        }),
    }
}

fn np_var(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let std_val = np_std(args)?;
    match std_val {
        Value::Float(f) => Ok(Value::Float(f * f)),
        _ => Err(RuntimeError { message: "Internal error".to_string() }),
    }
}

fn np_min(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "min() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            if data.is_empty() {
                return Ok(Value::Float(f64::NAN));
            }
            Ok(Value::Float(data.iter().cloned().fold(f64::INFINITY, f64::min)))
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(Value::Float(f64::NAN));
            }
            let min_val = arr.iter()
                .filter_map(|v| match v {
                    Value::Integer(i) => Some(*i as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                })
                .fold(f64::INFINITY, f64::min);
            Ok(Value::Float(min_val))
        }
        _ => Err(RuntimeError {
            message: "min() requires an array argument".to_string(),
        }),
    }
}

fn np_max(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "max() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            if data.is_empty() {
                return Ok(Value::Float(f64::NAN));
            }
            Ok(Value::Float(data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(Value::Float(f64::NAN));
            }
            let max_val = arr.iter()
                .filter_map(|v| match v {
                    Value::Integer(i) => Some(*i as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                })
                .fold(f64::NEG_INFINITY, f64::max);
            Ok(Value::Float(max_val))
        }
        _ => Err(RuntimeError {
            message: "max() requires an array argument".to_string(),
        }),
    }
}

fn np_argmin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "argmin() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            if data.is_empty() {
                return Ok(Value::Integer(-1));
            }
            let (min_idx, _) = data.iter().enumerate()
                .fold((0, f64::INFINITY), |(min_i, min_v), (i, &v)| {
                    if v < min_v { (i, v) } else { (min_i, min_v) }
                });
            Ok(Value::Integer(min_idx as i64))
        }
        _ => Err(RuntimeError {
            message: "argmin() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_argmax(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "argmax() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            if data.is_empty() {
                return Ok(Value::Integer(-1));
            }
            let (max_idx, _) = data.iter().enumerate()
                .fold((0, f64::NEG_INFINITY), |(max_i, max_v), (i, &v)| {
                    if v > max_v { (i, v) } else { (max_i, max_v) }
                });
            Ok(Value::Integer(max_idx as i64))
        }
        _ => Err(RuntimeError {
            message: "argmax() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_cumsum(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "cumsum() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let mut cumsum = Vec::with_capacity(data.len());
            let mut sum = 0.0;
            for &x in &data {
                sum += x;
                cumsum.push(sum);
            }
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![cumsum.len()], cumsum).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "cumsum() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_any(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "any() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            Ok(Value::Bool(data.iter().any(|&x| x != 0.0)))
        }
        _ => Err(RuntimeError {
            message: "any() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_all(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "all() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            Ok(Value::Bool(data.iter().all(|&x| x != 0.0)))
        }
        _ => Err(RuntimeError {
            message: "all() requires an ndarray argument".to_string(),
        }),
    }
}

// ============================================================================
// Linear Algebra Functions
// ============================================================================

fn np_dot(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "dot() requires two array arguments".to_string(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::NdArray(a), Value::NdArray(b)) => {
            let data_a = a.to_f64_vec()?;
            let data_b = b.to_f64_vec()?;
            
            if data_a.len() != data_b.len() {
                return Err(RuntimeError {
                    message: "dot() requires arrays of equal length".to_string(),
                });
            }

            let result: f64 = data_a.iter().zip(data_b.iter()).map(|(&x, &y)| x * y).sum();
            Ok(Value::Float(result))
        }
        _ => Err(RuntimeError {
            message: "dot() requires ndarray arguments".to_string(),
        }),
    }
}

fn np_inner(args: Vec<Value>) -> Result<Value, RuntimeError> {
    np_dot(args)
}

fn np_outer(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "outer() requires two array arguments".to_string(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::NdArray(a), Value::NdArray(b)) => {
            let data_a = a.to_f64_vec()?;
            let data_b = b.to_f64_vec()?;
            
            let n = data_a.len();
            let m = data_b.len();
            let mut result = vec![0.0f64; n * m];
            
            for i in 0..n {
                for j in 0..m {
                    result[i * m + j] = data_a[i] * data_b[j];
                }
            }

            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n, m], result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "outer() requires ndarray arguments".to_string(),
        }),
    }
}

fn np_matmul(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "matmul() requires two 2D array arguments".to_string(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::NdArray(a), Value::NdArray(b)) => {
            let shape_a = a.shape();
            let shape_b = b.shape();
            
            if shape_a.len() != 2 || shape_b.len() != 2 {
                return Err(RuntimeError {
                    message: "matmul() requires 2D arrays".to_string(),
                });
            }

            if shape_a[1] != shape_b[0] {
                return Err(RuntimeError {
                    message: format!("matmul() incompatible shapes: {:?} and {:?}", shape_a, shape_b),
                });
            }

            let data_a = a.to_f64_vec()?;
            let data_b = b.to_f64_vec()?;
            
            let n = shape_a[0];
            let k = shape_a[1];
            let m = shape_b[1];
            let mut result = vec![0.0f64; n * m];
            
            for i in 0..n {
                for j in 0..m {
                    let mut sum = 0.0;
                    for p in 0..k {
                        sum += data_a[i * k + p] * data_b[p * m + j];
                    }
                    result[i * m + j] = sum;
                }
            }

            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n, m], result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "matmul() requires ndarray arguments".to_string(),
        }),
    }
}

fn np_trace(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "trace() requires a 2D array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let shape = arr.shape();
            if shape.len() != 2 {
                return Err(RuntimeError {
                    message: "trace() requires a 2D array".to_string(),
                });
            }

            let data = arr.to_f64_vec()?;
            let n = shape[0].min(shape[1]);
            let mut trace = 0.0;
            for i in 0..n {
                trace += data[i * shape[1] + i];
            }
            Ok(Value::Float(trace))
        }
        _ => Err(RuntimeError {
            message: "trace() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_diag(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "diag() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let shape = arr.shape();
            let data = arr.to_f64_vec()?;

            if shape.len() == 1 {
                // 1D to 2D diagonal matrix
                let n = shape[0];
                let mut result = vec![0.0f64; n * n];
                for i in 0..n {
                    result[i * n + i] = data[i];
                }
                let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n, n], result).unwrap());
                Ok(Value::NdArray(ndarray))
            } else if shape.len() == 2 {
                // 2D to 1D diagonal
                let n = shape[0].min(shape[1]);
                let mut result = Vec::with_capacity(n);
                for i in 0..n {
                    result.push(data[i * shape[1] + i]);
                }
                let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![n], result).unwrap());
                Ok(Value::NdArray(ndarray))
            } else {
                Err(RuntimeError {
                    message: "diag() requires 1D or 2D array".to_string(),
                })
            }
        }
        _ => Err(RuntimeError {
            message: "diag() requires an ndarray argument".to_string(),
        }),
    }
}

// ============================================================================
// Sorting Functions
// ============================================================================

fn np_sort(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "sort() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let mut data = arr.to_f64_vec()?;
            data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![data.len()], data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "sort() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_argsort(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "argsort() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let mut indices: Vec<usize> = (0..data.len()).collect();
            indices.sort_by(|&i, &j| data[i].partial_cmp(&data[j]).unwrap_or(std::cmp::Ordering::Equal));
            
            let result: Vec<Value> = indices.iter().map(|&i| Value::Integer(i as i64)).collect();
            Ok(Value::Array(result))
        }
        _ => Err(RuntimeError {
            message: "argsort() requires an ndarray argument".to_string(),
        }),
    }
}

fn np_where(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 3 {
        return Err(RuntimeError {
            message: "where() requires condition, x, and y arguments".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(cond) => {
            let cond_data = match cond {
                NdArray::Bool(b) => b.iter().copied().collect::<Vec<bool>>(),
                NdArray::F64(f) => f.iter().map(|&x| x != 0.0).collect(),
                NdArray::I64(i) => i.iter().map(|&x| x != 0).collect(),
            };

            let x_val = value_to_f64(&args[1])?;
            let y_val = value_to_f64(&args[2])?;

            let result: Vec<f64> = cond_data.iter()
                .map(|&c| if c { x_val } else { y_val })
                .collect();

            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![result.len()], result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "where() requires an ndarray condition".to_string(),
        }),
    }
}

// ============================================================================
// Set Operations
// ============================================================================

fn np_unique(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "unique() requires an array argument".to_string(),
        });
    }

    match &args[0] {
        Value::NdArray(arr) => {
            let data = arr.to_f64_vec()?;
            let mut unique_data = data.clone();
            unique_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            unique_data.dedup();
            
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![unique_data.len()], unique_data).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "unique() requires an ndarray argument".to_string(),
        }),
    }
}

// ============================================================================
// Logical Operations
// ============================================================================

fn np_logical_and(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "logical_and() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a != 0.0 && b != 0.0 { 1.0 } else { 0.0 })
}

fn np_logical_or(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "logical_or() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a != 0.0 || b != 0.0 { 1.0 } else { 0.0 })
}

fn np_logical_not(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError {
            message: "logical_not() requires an array argument".to_string(),
        });
    }
    unary_op(&args[0], |x| if x == 0.0 { 1.0 } else { 0.0 })
}

fn np_greater(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "greater() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a > b { 1.0 } else { 0.0 })
}

fn np_less(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "less() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a < b { 1.0 } else { 0.0 })
}

fn np_equal(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "equal() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a == b { 1.0 } else { 0.0 })
}

fn np_not_equal(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "not_equal() requires two array arguments".to_string(),
        });
    }
    element_wise_op(&args[0], &args[1], |a, b| if a != b { 1.0 } else { 0.0 })
}

// ============================================================================
// Random Functions
// ============================================================================

fn np_random(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Float(rand::random::<f64>()))
}

fn np_rand(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let shape = if args.is_empty() {
        vec![1]
    } else {
        match &args[0] {
            Value::Array(s) => s.iter().filter_map(|v| match v {
                Value::Integer(i) => Some(*i as usize),
                _ => None,
            }).collect(),
            Value::Integer(n) => vec![*n as usize],
            _ => vec![1],
        }
    };

    let size = shape.iter().product();
    let data: Vec<f64> = (0..size).map(|_| rand::random::<f64>()).collect();
    let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
    Ok(Value::NdArray(ndarray))
}

fn np_randint(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError {
            message: "randint() requires low and high arguments".to_string(),
        });
    }

    let low = value_to_i64(&args[0])?;
    let high = value_to_i64(&args[1])?;
    let shape = if args.len() >= 3 {
        match &args[2] {
            Value::Array(s) => s.iter().filter_map(|v| match v {
                Value::Integer(i) => Some(*i as usize),
                _ => None,
            }).collect(),
            Value::Integer(n) => vec![*n as usize],
            _ => vec![1],
        }
    } else {
        vec![1]
    };

    let size = shape.iter().product();
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let data: Vec<f64> = (0..size).map(|_| rng.gen_range(low..high) as f64).collect();
    let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, data).unwrap());
    Ok(Value::NdArray(ndarray))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn value_to_f64(v: &Value) -> Result<f64, RuntimeError> {
    match v {
        Value::Integer(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {} to float", v.type_name()),
        }),
    }
}

fn value_to_i64(v: &Value) -> Result<i64, RuntimeError> {
    match v {
        Value::Integer(i) => Ok(*i),
        Value::Float(f) => Ok(*f as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {} to int", v.type_name()),
        }),
    }
}

fn unary_op<F>(arr: &Value, op: F) -> Result<Value, RuntimeError>
where
    F: Fn(f64) -> f64,
{
    match arr {
        Value::NdArray(a) => {
            let data = a.to_f64_vec()?;
            let result: Vec<f64> = data.iter().map(|&x| op(x)).collect();
            let shape = a.shape().clone();
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "Operation requires an ndarray argument".to_string(),
        }),
    }
}

fn element_wise_op<F>(a: &Value, b: &Value, op: F) -> Result<Value, RuntimeError>
where
    F: Fn(f64, f64) -> f64,
{
    match (a, b) {
        (Value::NdArray(arr_a), Value::NdArray(arr_b)) => {
            let data_a = arr_a.to_f64_vec()?;
            let data_b = arr_b.to_f64_vec()?;

            // Handle broadcasting (simplified: scalar + array)
            if data_a.len() == 1 {
                let scalar = data_a[0];
                let result: Vec<f64> = data_b.iter().map(|&x| op(scalar, x)).collect();
                let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![result.len()], result).unwrap());
                return Ok(Value::NdArray(ndarray));
            }
            if data_b.len() == 1 {
                let scalar = data_b[0];
                let result: Vec<f64> = data_a.iter().map(|&x| op(x, scalar)).collect();
                let ndarray = NdArray::F64(ArrayD::from_shape_vec(vec![result.len()], result).unwrap());
                return Ok(Value::NdArray(ndarray));
            }

            if data_a.len() != data_b.len() {
                return Err(RuntimeError {
                    message: format!("Arrays must have same length: {} vs {}", data_a.len(), data_b.len()),
                });
            }

            let result: Vec<f64> = data_a.iter().zip(data_b.iter()).map(|(&x, &y)| op(x, y)).collect();
            let shape = arr_a.shape().clone();
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        (Value::NdArray(arr), scalar) | (scalar, Value::NdArray(arr)) => {
            let s = value_to_f64(scalar)?;
            let data = arr.to_f64_vec()?;
            let result: Vec<f64> = data.iter().map(|&x| op(x, s)).collect();
            let shape = arr.shape().clone();
            let ndarray = NdArray::F64(ArrayD::from_shape_vec(shape, result).unwrap());
            Ok(Value::NdArray(ndarray))
        }
        _ => Err(RuntimeError {
            message: "Operation requires ndarray arguments".to_string(),
        }),
    }
}
