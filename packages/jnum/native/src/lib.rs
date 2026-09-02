use rand::Rng;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const TYPE_NAME: &str = "ndarray";

#[derive(Clone, Debug)]
struct Array {
    shape: Vec<usize>,
    data: Vec<f64>,
}

impl Array {
    fn new(shape: Vec<usize>, data: Vec<f64>) -> Result<Self, String> {
        let expected = shape
            .iter()
            .try_fold(1usize, |size, value| size.checked_mul(*value))
            .ok_or("array shape is too large")?;
        if expected != data.len() {
            return Err(format!(
                "shape {shape:?} requires {expected} values, got {}",
                data.len()
            ));
        }
        Ok(Self { shape, data })
    }

    fn vector(data: Vec<f64>) -> Self {
        Self {
            shape: vec![data.len()],
            data,
        }
    }
}

#[derive(Default)]
struct Store {
    next: u64,
    arrays: HashMap<u64, Array>,
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(Store {
            next: 1,
            arrays: HashMap::new(),
        })
    })
}

fn put(array: Array) -> Result<Value, String> {
    let mut store = store().lock().map_err(|_| "array store is unavailable")?;
    let id = store.next;
    store.next = store
        .next
        .checked_add(1)
        .ok_or("array handle space exhausted")?;
    store.arrays.insert(id, array);
    Ok(ject_native::resource(id, TYPE_NAME))
}

fn resource_id(value: &Value) -> Option<u64> {
    value.get("$ject_resource")?.get("id")?.as_u64()
}

fn get(value: &Value) -> Result<Array, String> {
    let id = resource_id(value).ok_or("expected a JNUM array")?;
    store()
        .lock()
        .map_err(|_| "array store is unavailable")?
        .arrays
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("array handle {id} is no longer valid"))
}

fn number(value: &Value, label: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("{label} must be a number"))
}

fn integer(value: &Value, label: &str) -> Result<usize, String> {
    let value = value
        .as_i64()
        .ok_or_else(|| format!("{label} must be an integer"))?;
    usize::try_from(value).map_err(|_| format!("{label} must be non-negative"))
}

fn shape(value: &Value) -> Result<Vec<usize>, String> {
    if let Some(size) = value.as_i64() {
        return usize::try_from(size)
            .map(|size| vec![size])
            .map_err(|_| "shape must be non-negative".into());
    }
    value
        .as_array()
        .ok_or("shape must be an integer or array")?
        .iter()
        .map(|part| integer(part, "shape dimension"))
        .collect()
}

fn flatten_json(value: &Value, expected: &mut Option<Vec<usize>>) -> Result<Vec<f64>, String> {
    fn walk(value: &Value) -> Result<(Vec<usize>, Vec<f64>), String> {
        if let Some(number) = value.as_f64() {
            return Ok((Vec::new(), vec![number]));
        }
        if let Some(boolean) = value.as_bool() {
            return Ok((Vec::new(), vec![if boolean { 1.0 } else { 0.0 }]));
        }
        let values = value
            .as_array()
            .ok_or("array data must contain only numbers, booleans, or arrays")?;
        if values.is_empty() {
            return Ok((vec![0], Vec::new()));
        }
        let (child_shape, mut data) = walk(&values[0])?;
        for child in &values[1..] {
            let (next_shape, next_data) = walk(child)?;
            if next_shape != child_shape {
                return Err("nested arrays must be rectangular".into());
            }
            data.extend(next_data);
        }
        let mut result_shape = vec![values.len()];
        result_shape.extend(child_shape);
        Ok((result_shape, data))
    }
    let (found_shape, data) = walk(value)?;
    *expected = Some(found_shape);
    Ok(data)
}

fn array_arg(value: &Value) -> Result<Array, String> {
    if resource_id(value).is_some() {
        return get(value);
    }
    let mut found_shape = None;
    let data = flatten_json(value, &mut found_shape)?;
    Array::new(found_shape.unwrap_or_default(), data)
}

fn argument<'a>(args: &'a [Value], index: usize, name: &str) -> Result<&'a Value, String> {
    args.get(index)
        .ok_or_else(|| format!("missing required argument '{name}'"))
}

fn unary(args: &[Value], name: &str, mut op: impl FnMut(f64) -> f64) -> Result<Value, String> {
    let mut array = array_arg(argument(args, 0, "value")?)?;
    for value in &mut array.data {
        *value = op(*value);
    }
    put(array).map_err(|error| format!("{name}: {error}"))
}

fn paired(left: &Array, right: &Array, op: impl Fn(f64, f64) -> f64) -> Result<Array, String> {
    if left.shape != right.shape {
        return Err(format!(
            "array shapes differ: {:?} and {:?}",
            left.shape, right.shape
        ));
    }
    Array::new(
        left.shape.clone(),
        left.data
            .iter()
            .zip(&right.data)
            .map(|(a, b)| op(*a, *b))
            .collect(),
    )
}

fn binary(args: &[Value], op: impl Fn(f64, f64) -> f64) -> Result<Value, String> {
    let left = array_arg(argument(args, 0, "left")?)?;
    let right = array_arg(argument(args, 1, "right")?)?;
    put(paired(&left, &right, op)?)
}

fn reduction(args: &[Value], name: &str) -> Result<Value, String> {
    let array = array_arg(argument(args, 0, "value")?)?;
    if array.data.is_empty() && !matches!(name, "sum" | "any" | "all") {
        return Err(format!("{name} requires a non-empty array"));
    }
    let value = match name {
        "sum" => json!(array.data.iter().sum::<f64>()),
        "mean" => json!(array.data.iter().sum::<f64>() / array.data.len() as f64),
        "var" | "std" => {
            let mean = array.data.iter().sum::<f64>() / array.data.len() as f64;
            let variance = array
                .data
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / array.data.len() as f64;
            json!(if name == "std" {
                variance.sqrt()
            } else {
                variance
            })
        }
        "min" => json!(array.data.iter().copied().fold(f64::INFINITY, f64::min)),
        "max" => json!(array.data.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        "argmin" => json!(array
            .data
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|v| v.0)
            .unwrap_or(0)),
        "argmax" => json!(array
            .data
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|v| v.0)
            .unwrap_or(0)),
        "any" => json!(array.data.iter().any(|value| *value != 0.0)),
        "all" => json!(array.data.iter().all(|value| *value != 0.0)),
        _ => return Err(format!("unknown reduction '{name}'")),
    };
    Ok(value)
}

fn create(function: &str, args: &[Value]) -> Result<Value, String> {
    match function {
        "array" => put(array_arg(argument(args, 0, "data")?)?),
        "zeros" | "ones" => {
            let shape = shape(argument(args, 0, "shape")?)?;
            let size = shape.iter().product();
            put(Array::new(
                shape,
                vec![if function == "ones" { 1.0 } else { 0.0 }; size],
            )?)
        }
        "arange" => {
            let start = number(argument(args, 0, "start")?, "start")?;
            let stop = number(argument(args, 1, "stop")?, "stop")?;
            let step = number(argument(args, 2, "step")?, "step")?;
            if step == 0.0 {
                return Err("step must not be zero".into());
            }
            let mut data = Vec::new();
            let mut value = start;
            while (step > 0.0 && value < stop) || (step < 0.0 && value > stop) {
                data.push(value);
                value += step;
            }
            put(Array::vector(data))
        }
        "linspace" => {
            let start = number(argument(args, 0, "start")?, "start")?;
            let stop = number(argument(args, 1, "stop")?, "stop")?;
            let count = integer(argument(args, 2, "count")?, "count")?;
            let data = (0..count)
                .map(|index| {
                    if count <= 1 {
                        start
                    } else {
                        start + (stop - start) * index as f64 / (count - 1) as f64
                    }
                })
                .collect();
            put(Array::vector(data))
        }
        "eye" => {
            let rows = integer(argument(args, 0, "rows")?, "rows")?;
            let columns = integer(argument(args, 1, "columns")?, "columns")?;
            let mut data = vec![0.0; rows * columns];
            for index in 0..rows.min(columns) {
                data[index * columns + index] = 1.0;
            }
            put(Array::new(vec![rows, columns], data)?)
        }
        _ => Err(format!("unknown constructor '{function}'")),
    }
}

fn manipulate(function: &str, args: &[Value]) -> Result<Value, String> {
    match function {
        "reshape" => {
            let array = array_arg(argument(args, 0, "value")?)?;
            put(Array::new(shape(argument(args, 1, "shape")?)?, array.data)?)
        }
        "flatten" => put(Array::vector(array_arg(argument(args, 0, "value")?)?.data)),
        "transpose" => {
            let array = array_arg(argument(args, 0, "value")?)?;
            if array.shape.len() != 2 {
                return Err("transpose currently requires a 2D array".into());
            }
            let (rows, columns) = (array.shape[0], array.shape[1]);
            let mut data = vec![0.0; array.data.len()];
            for row in 0..rows {
                for column in 0..columns {
                    data[column * rows + row] = array.data[row * columns + column];
                }
            }
            put(Array::new(vec![columns, rows], data)?)
        }
        "concatenate" | "stack" => {
            let values = argument(args, 0, "values")?
                .as_array()
                .ok_or("values must be an array of JNUM arrays")?;
            let arrays = values
                .iter()
                .map(array_arg)
                .collect::<Result<Vec<_>, _>>()?;
            if arrays.is_empty() {
                return Err("values must not be empty".into());
            }
            if function == "concatenate" {
                let tail = &arrays[0].shape[1..];
                if arrays.iter().any(|array| &array.shape[1..] != tail) {
                    return Err("array shapes are incompatible for concatenation".into());
                }
                let mut shape = arrays[0].shape.clone();
                shape[0] = arrays.iter().map(|array| array.shape[0]).sum();
                put(Array::new(
                    shape,
                    arrays.into_iter().flat_map(|array| array.data).collect(),
                )?)
            } else {
                if arrays.iter().any(|array| array.shape != arrays[0].shape) {
                    return Err("array shapes are incompatible for stacking".into());
                }
                let mut shape = vec![arrays.len()];
                shape.extend(arrays[0].shape.clone());
                put(Array::new(
                    shape,
                    arrays.into_iter().flat_map(|array| array.data).collect(),
                )?)
            }
        }
        _ => Err(format!("unknown manipulation '{function}'")),
    }
}

fn linear(function: &str, args: &[Value]) -> Result<Value, String> {
    let left = array_arg(argument(args, 0, "left")?)?;
    match function {
        "dot" => {
            let right = array_arg(argument(args, 1, "right")?)?;
            if left.data.len() != right.data.len() {
                return Err("dot requires equal-sized arrays".into());
            }
            Ok(json!(left
                .data
                .iter()
                .zip(&right.data)
                .map(|(a, b)| a * b)
                .sum::<f64>()))
        }
        "outer" => {
            let right = array_arg(argument(args, 1, "right")?)?;
            let data = left
                .data
                .iter()
                .flat_map(|a| right.data.iter().map(move |b| a * b))
                .collect();
            put(Array::new(vec![left.data.len(), right.data.len()], data)?)
        }
        "matmul" => {
            let right = array_arg(argument(args, 1, "right")?)?;
            if left.shape.len() != 2 || right.shape.len() != 2 || left.shape[1] != right.shape[0] {
                return Err("matmul requires compatible 2D arrays".into());
            }
            let (rows, inner, columns) = (left.shape[0], left.shape[1], right.shape[1]);
            let mut data = vec![0.0; rows * columns];
            for row in 0..rows {
                for column in 0..columns {
                    for index in 0..inner {
                        data[row * columns + column] +=
                            left.data[row * inner + index] * right.data[index * columns + column];
                    }
                }
            }
            put(Array::new(vec![rows, columns], data)?)
        }
        "trace" => {
            if left.shape.len() != 2 {
                return Err("trace requires a 2D array".into());
            }
            Ok(json!((0..left.shape[0].min(left.shape[1]))
                .map(|i| left.data[i * left.shape[1] + i])
                .sum::<f64>()))
        }
        "diag" => {
            if left.shape.len() == 1 {
                let size = left.data.len();
                let mut data = vec![0.0; size * size];
                for index in 0..size {
                    data[index * size + index] = left.data[index];
                }
                put(Array::new(vec![size, size], data)?)
            } else if left.shape.len() == 2 {
                let data = (0..left.shape[0].min(left.shape[1]))
                    .map(|i| left.data[i * left.shape[1] + i])
                    .collect();
                put(Array::vector(data))
            } else {
                Err("diag requires a 1D or 2D array".into())
            }
        }
        _ => Err(format!("unknown linear algebra operation '{function}'")),
    }
}

fn call(function: &str, args: Vec<Value>) -> Result<Value, String> {
    match function {
        "__drop_resource" => {
            if let Some(id) = args.first().and_then(Value::as_u64) {
                store()
                    .lock()
                    .map_err(|_| "array store is unavailable")?
                    .arrays
                    .remove(&id);
            }
            Ok(Value::Null)
        }
        "array" | "zeros" | "ones" | "arange" | "linspace" | "eye" => create(function, &args),
        "shape" => Ok(json!(array_arg(argument(&args, 0, "value")?)?.shape)),
        "ndim" => Ok(json!(array_arg(argument(&args, 0, "value")?)?.shape.len())),
        "size" => Ok(json!(array_arg(argument(&args, 0, "value")?)?.data.len())),
        "dtype" => {
            let _ = array_arg(argument(&args, 0, "value")?)?;
            Ok(json!("float64"))
        }
        "to_array" => Ok(json!(array_arg(argument(&args, 0, "value")?)?.data)),
        "reshape" | "flatten" | "transpose" | "concatenate" | "stack" => {
            manipulate(function, &args)
        }
        "sqrt" => unary(&args, function, f64::sqrt),
        "exp" => unary(&args, function, f64::exp),
        "log" => unary(&args, function, f64::ln),
        "log10" => unary(&args, function, f64::log10),
        "abs" => unary(&args, function, f64::abs),
        "ceil" => unary(&args, function, f64::ceil),
        "floor" => unary(&args, function, f64::floor),
        "round" => unary(&args, function, f64::round),
        "sin" => unary(&args, function, f64::sin),
        "cos" => unary(&args, function, f64::cos),
        "tan" => unary(&args, function, f64::tan),
        "arcsin" => unary(&args, function, f64::asin),
        "arccos" => unary(&args, function, f64::acos),
        "arctan" => unary(&args, function, f64::atan),
        "degrees" => unary(&args, function, f64::to_degrees),
        "radians" => unary(&args, function, f64::to_radians),
        "sinh" => unary(&args, function, f64::sinh),
        "cosh" => unary(&args, function, f64::cosh),
        "tanh" => unary(&args, function, f64::tanh),
        "clip" => {
            let min = number(argument(&args, 1, "minimum")?, "minimum")?;
            let max = number(argument(&args, 2, "maximum")?, "maximum")?;
            unary(&args, function, |value| value.clamp(min, max))
        }
        "arctan2" => binary(&args, f64::atan2),
        "sum" | "mean" | "std" | "var" | "min" | "max" | "argmin" | "argmax" | "any" | "all" => {
            reduction(&args, function)
        }
        "cumsum" => {
            let mut total = 0.0;
            unary(&args, function, |value| {
                total += value;
                total
            })
        }
        "dot" | "outer" | "matmul" | "trace" | "diag" => linear(function, &args),
        "sort" | "argsort" | "unique" => {
            let array = array_arg(argument(&args, 0, "value")?)?;
            if function == "argsort" {
                let mut indices: Vec<_> = (0..array.data.len()).collect();
                indices.sort_by(|a, b| array.data[*a].total_cmp(&array.data[*b]));
                put(Array::vector(
                    indices.into_iter().map(|v| v as f64).collect(),
                ))
            } else {
                let mut data = array.data;
                data.sort_by(f64::total_cmp);
                if function == "unique" {
                    data.dedup_by(|a, b| a.total_cmp(b) == Ordering::Equal);
                }
                put(Array::vector(data))
            }
        }
        "logical_and" => binary(&args, |a, b| if a != 0.0 && b != 0.0 { 1.0 } else { 0.0 }),
        "logical_or" => binary(&args, |a, b| if a != 0.0 || b != 0.0 { 1.0 } else { 0.0 }),
        "logical_not" => unary(&args, function, |a| if a == 0.0 { 1.0 } else { 0.0 }),
        "greater" => binary(&args, |a, b| if a > b { 1.0 } else { 0.0 }),
        "less" => binary(&args, |a, b| if a < b { 1.0 } else { 0.0 }),
        "equal" => binary(&args, |a, b| if a == b { 1.0 } else { 0.0 }),
        "not_equal" => binary(&args, |a, b| if a != b { 1.0 } else { 0.0 }),
        "where" => {
            let condition = array_arg(argument(&args, 0, "condition")?)?;
            let yes = array_arg(argument(&args, 1, "yes")?)?;
            let no = array_arg(argument(&args, 2, "no")?)?;
            if condition.shape != yes.shape || yes.shape != no.shape {
                return Err("where requires arrays with equal shapes".into());
            }
            put(Array::new(
                condition.shape,
                condition
                    .data
                    .iter()
                    .zip(yes.data)
                    .zip(no.data)
                    .map(|((c, y), n)| if *c != 0.0 { y } else { n })
                    .collect(),
            )?)
        }
        "random" => Ok(json!(rand::thread_rng().gen::<f64>())),
        "inf" => ject_native::special_float(f64::INFINITY).map_err(str::to_string),
        "nan" => ject_native::special_float(f64::NAN).map_err(str::to_string),
        "randint" => {
            let low = argument(&args, 0, "low")?
                .as_i64()
                .ok_or("low must be an integer")?;
            let high = argument(&args, 1, "high")?
                .as_i64()
                .ok_or("high must be an integer")?;
            if low >= high {
                return Err("low must be smaller than high".into());
            }
            let size = integer(argument(&args, 2, "size")?, "size")?;
            let mut rng = rand::thread_rng();
            put(Array::vector(
                (0..size).map(|_| rng.gen_range(low..high) as f64).collect(),
            ))
        }
        _ => Err(format!("unknown function '{function}'")),
    }
}

ject_native::ject_plugin!(
    "jnum",
    [
        "array",
        "zeros",
        "ones",
        "arange",
        "linspace",
        "eye",
        "shape",
        "ndim",
        "size",
        "dtype",
        "to_array",
        "reshape",
        "flatten",
        "transpose",
        "concatenate",
        "stack",
        "sqrt",
        "exp",
        "log",
        "log10",
        "abs",
        "ceil",
        "floor",
        "round",
        "clip",
        "sin",
        "cos",
        "tan",
        "arcsin",
        "arccos",
        "arctan",
        "arctan2",
        "degrees",
        "radians",
        "sinh",
        "cosh",
        "tanh",
        "sum",
        "mean",
        "std",
        "var",
        "min",
        "max",
        "argmin",
        "argmax",
        "cumsum",
        "any",
        "all",
        "dot",
        "outer",
        "matmul",
        "trace",
        "diag",
        "sort",
        "argsort",
        "where",
        "unique",
        "logical_and",
        "logical_or",
        "logical_not",
        "greater",
        "less",
        "equal",
        "not_equal",
        "random",
        "randint",
        "inf",
        "nan"
    ],
    call
);
