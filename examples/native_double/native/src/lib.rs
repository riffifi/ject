use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static COUNTERS: OnceLock<Mutex<HashMap<u64, i64>>> = OnceLock::new();

fn counters() -> &'static Mutex<HashMap<u64, i64>> {
    COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn resource_id(value: &Value) -> Result<u64, String> {
    value
        .get("$ject_resource")
        .and_then(|resource| resource.get("id"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "expected a native_double counter".to_string())
}

fn call(function: &str, args: Vec<Value>) -> Result<Value, String> {
    match function {
        "double" => {
            let value = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| "double expects one integer".to_string())?;
            Ok(json!(value * 2))
        }
        "new_counter" => {
            let initial = args.first().and_then(Value::as_i64).unwrap_or(0);
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            counters()
                .lock()
                .map_err(|_| "counter lock poisoned")?
                .insert(id, initial);
            Ok(ject_native::resource(id, "native_counter"))
        }
        "increment" => {
            let id = resource_id(args.first().ok_or("increment expects a counter")?)?;
            let mut counters = counters().lock().map_err(|_| "counter lock poisoned")?;
            let value = counters.get_mut(&id).ok_or("counter was already closed")?;
            *value += 1;
            Ok(json!(*value))
        }
        "__drop_resource" => {
            if let Some(id) = args.first().and_then(Value::as_u64) {
                if let Ok(mut counters) = counters().lock() {
                    counters.remove(&id);
                }
            }
            Ok(Value::Null)
        }
        _ => Err(format!("unknown function '{function}'")),
    }
}

ject_native::ject_plugin!(
    "native_double",
    ["double", "new_counter", "increment"],
    call
);
