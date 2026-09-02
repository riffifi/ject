//! Generalized native-extension system.
//!
//! Before this module, the interpreter had numeric and GUI modules special-cased directly:
//! `Value::NdArray(NdArray)` was a hardcoded variant in the core `Value` enum,
//! `is_native_only_module()` hardcoded `"base" | "jgui" | "jnum"`, and function
//! dispatch sniffed a `"np_"`/`"gui_"` prefix on the function name string to decide
//! which Rust backend to call. None of that generalizes to a third native
//! extension without editing the interpreter itself.
//!
//! This module replaces all of that with two traits:
//!
//! - [`NativeObject`]: any Rust type that needs to flow through the interpreter as
//!   a first-class value (like jnum's `NdArray`) implements this and is wrapped in
//!   `Value::Native`, instead of getting its own hardcoded `Value` variant.
//! - [`NativeModule`]: any built-in native module implements
//!   this once and registers itself; the interpreter and module loader consult the
//!   registry generically instead of special-casing module names.

use crate::interpreter::RuntimeError;
use crate::value::Value;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

struct CallbackContext {
    interpreter: *mut crate::interpreter::Interpreter,
    callbacks: Vec<Value>,
    module: Option<String>,
    descriptor: Option<ExternalDescriptor>,
}

thread_local! {
    static CALLBACK_CONTEXTS: RefCell<Vec<CallbackContext>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy)]
enum ExternalDescriptor {
    V1(&'static ject_native::PluginV1),
    V2(&'static ject_native::PluginV2),
}

unsafe extern "C" fn host_call_callback(
    id: u64,
    arguments_ptr: *const u8,
    arguments_len: usize,
) -> ject_native::Buffer {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_host_callback(id, arguments_ptr, arguments_len)
    }))
    .unwrap_or_else(|_| Err("Ject callback panicked".to_string()));
    let envelope = match result {
        Ok(value) => serde_json::json!({ "ok": value }),
        Err(error) => serde_json::json!({ "error": error }),
    };
    ject_native::Buffer::from_vec(
        serde_json::to_vec(&envelope).expect("callback envelope is serializable"),
    )
}

static HOST_API: ject_native::HostV1 = ject_native::HostV1 {
    call_callback: host_call_callback,
    free_buffer: ject_native::free_buffer,
};

/// A value backed by native (Rust) code. Implement this for any type a native
/// extension wants to hand back into Ject as a value (jnum's `NdArray`, for
/// example) instead of adding a new hardcoded variant to `Value`.
pub trait NativeObject: fmt::Debug {
    /// Name shown by `type_of()` and in error messages, e.g. `"ndarray"`.
    fn type_name(&self) -> &str;

    /// Human-readable representation, used by `print` and string interpolation.
    fn display(&self) -> String;

    /// Structural equality against another native object, which may be a
    /// different concrete type entirely (in which case this should be `false`).
    /// Default: never equal. Override for real equality (see `NdArray`'s impl).
    fn native_eq(&self, other: &dyn NativeObject) -> bool {
        let _ = other;
        false
    }

    /// For downcasting back to the concrete type inside the extension that
    /// created it, e.g. `value.as_any().downcast_ref::<NdArray>()`.
    fn as_any(&self) -> &dyn Any;
}

/// Wraps `Rc<dyn NativeObject>` so it can implement `PartialEq`/`Clone`/`Debug`
/// the way `Value`'s derive needs -- a bare `Rc<dyn NativeObject>` can't derive
/// `PartialEq` since trait objects don't get one for free.
#[derive(Clone, Debug)]
pub struct NativeValue(pub Rc<dyn NativeObject>);

impl PartialEq for NativeValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.native_eq(&*other.0)
    }
}

impl NativeValue {
    pub fn new(obj: impl NativeObject + 'static) -> Self {
        NativeValue(Rc::new(obj))
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }
}

/// A pluggable native module -- the generalized replacement for hardcoding a
/// specific module name into the interpreter's module resolution and call
/// dispatch. Implement this once per native extension and register it in
/// [`build_registry`].
///
/// Requires `Send + Sync` since the registry is a single process-wide static (see
/// `native_registry`) -- native modules are expected to be stateless dispatchers,
/// not holders of interior state, so this should be free for real implementations.
pub trait NativeModule: Send + Sync {
    /// The name used in `import "name"`.
    fn name(&self) -> &str;

    /// Everything this module exports: functions and constants. Functions should
    /// be `Value::NativeFunction { module: self.name(), name: ... }`.
    fn exports(&self) -> HashMap<String, Value>;

    /// Calls one of this module's functions by name (without any module prefix).
    fn call(&self, fn_name: &str, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

/// Registry of all native modules available to the interpreter.
pub struct NativeRegistry {
    modules: HashMap<String, Box<dyn NativeModule>>,
}

impl NativeRegistry {
    fn new() -> Self {
        NativeRegistry {
            modules: HashMap::new(),
        }
    }

    fn register(&mut self, module: Box<dyn NativeModule>) -> Result<(), String> {
        let name = module.name().to_string();
        if self.modules.contains_key(&name) {
            return Err(format!("native module '{name}' is already registered"));
        }
        self.modules.insert(name, module);
        Ok(())
    }

    fn replace(&mut self, module: Box<dyn NativeModule>) {
        self.modules.insert(module.name().to_string(), module);
    }

    pub fn get(&self, name: &str) -> Option<&dyn NativeModule> {
        self.modules.get(name).map(|m| m.as_ref())
    }
}

/// The process-wide registry of native modules. Native modules are stateless
/// Rust-backed extensions (not per-script state), so one shared registry -- built
/// once, lazily, on first use -- is simpler than threading a registry through
/// every `Interpreter`.
fn native_registry() -> &'static std::sync::RwLock<NativeRegistry> {
    static REGISTRY: std::sync::OnceLock<std::sync::RwLock<NativeRegistry>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = NativeRegistry::new();
        registry
            .register(Box::new(crate::jnum::JnumModule))
            .expect("unique built-in module");
        registry
            .register(Box::new(crate::jgui::JguiModule))
            .expect("unique built-in module");
        std::sync::RwLock::new(registry)
    })
}

pub fn module_exports(name: &str) -> Option<HashMap<String, Value>> {
    native_registry()
        .read()
        .ok()?
        .get(name)
        .map(NativeModule::exports)
}

pub fn call_module(module: &str, function: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let registry = native_registry().read().map_err(|_| RuntimeError {
        message: "native module registry is poisoned".to_string(),
    })?;
    let module_impl = registry.get(module).ok_or_else(|| RuntimeError {
        message: format!("Native module '{module}' is not registered"),
    })?;
    module_impl.call(function, args)
}

pub fn call_module_with_interpreter(
    module: &str,
    function: &str,
    args: Vec<Value>,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<Value, RuntimeError> {
    CALLBACK_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().push(CallbackContext {
            interpreter,
            callbacks: Vec::new(),
            module: None,
            descriptor: None,
        });
    });
    let result = call_module(module, function, args);
    CALLBACK_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().pop();
    });
    result
}

fn invoke_host_callback(
    id: u64,
    arguments_ptr: *const u8,
    arguments_len: usize,
) -> Result<serde_json::Value, String> {
    if arguments_ptr.is_null() && arguments_len != 0 {
        return Err("plugin passed an invalid callback buffer".to_string());
    }
    let bytes = if arguments_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(arguments_ptr, arguments_len) }
    };
    let arguments: Vec<serde_json::Value> = serde_json::from_slice(bytes)
        .map_err(|error| format!("plugin passed invalid callback arguments: {error}"))?;
    let (interpreter, callback, module, descriptor) = CALLBACK_CONTEXTS.with(|contexts| {
        let contexts = contexts.borrow();
        let context = contexts
            .last()
            .ok_or_else(|| "no Ject callback context is active".to_string())?;
        let callback = id
            .checked_sub(1)
            .and_then(|index| context.callbacks.get(index as usize))
            .cloned()
            .ok_or_else(|| format!("unknown Ject callback handle {id}"))?;
        Ok::<_, String>((
            context.interpreter,
            callback,
            context.module.clone().unwrap_or_default(),
            context.descriptor,
        ))
    })?;
    let descriptor = descriptor.ok_or_else(|| "native callback ABI is not active".to_string())?;
    let arguments = arguments
        .into_iter()
        .map(|value| json_to_value(value, &module, descriptor))
        .collect::<Result<Vec<_>, _>>()?;
    let value = unsafe { &mut *interpreter }
        .invoke_callable(&callback, arguments)
        .map_err(|error| error.message)?;
    value_to_json(&value, &module, Some(descriptor))
}

/// Loads a versioned `ject-native-1` dynamic library. The library owns its
/// descriptor and remains loaded for the rest of the process.
pub fn register_dynamic(
    path: &Path,
    expected_name: Option<&str>,
    expected_abi: Option<&str>,
) -> Result<String, String> {
    let module = unsafe { DynamicNativeModule::load(path)? };
    let name = module.name.clone();
    if let Some(expected) = expected_name {
        if expected != name {
            return Err(format!(
                "native artifact declares module '{name}', expected '{expected}'"
            ));
        }
    }
    if let Some(expected) = expected_abi {
        let actual = format!("ject-native-{}", module.abi_version);
        if expected != actual {
            return Err(format!(
                "native artifact uses {actual}, but Ject.toml declares {expected}"
            ));
        }
    }
    let mut registry = native_registry()
        .write()
        .map_err(|_| "native module registry is poisoned".to_string())?;
    // An explicitly installed package is authoritative over a bundled compatibility
    // backend with the same name. This is what lets jgui/jnum evolve as ordinary
    // mixed packages without requiring a new Ject executable for every release.
    registry.replace(Box::new(module));
    Ok(name)
}

struct DynamicNativeModule {
    name: String,
    abi_version: u32,
    exports: Vec<String>,
    descriptor: ExternalDescriptor,
    _library: libloading::Library,
}

#[derive(Debug)]
struct ExternalResource {
    module: String,
    type_name: String,
    id: u64,
    descriptor: ExternalDescriptor,
}

impl Drop for ExternalResource {
    fn drop(&mut self) {
        unsafe {
            match self.descriptor {
                ExternalDescriptor::V1(descriptor) => (descriptor.drop_resource)(self.id),
                ExternalDescriptor::V2(descriptor) => (descriptor.drop_resource)(self.id),
            }
        };
    }
}

impl NativeObject for ExternalResource {
    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn display(&self) -> String {
        format!("<{} {}::{}>", self.type_name, self.module, self.id)
    }

    fn native_eq(&self, other: &dyn NativeObject) -> bool {
        other
            .as_any()
            .downcast_ref::<ExternalResource>()
            .is_some_and(|other| self.module == other.module && self.id == other.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DynamicNativeModule {
    unsafe fn load(path: &Path) -> Result<Self, String> {
        let library = unsafe { libloading::Library::new(path) }
            .map_err(|e| format!("failed to load {}: {e}", path.display()))?;
        let descriptor = if let Ok(entry) =
            unsafe { library.get::<ject_native::EntryFnV2>(ject_native::ENTRY_SYMBOL_V2) }
        {
            let descriptor = unsafe { entry() }
                .as_ref()
                .ok_or_else(|| "plugin returned a null v2 descriptor".to_string())?;
            if descriptor.abi_version != ject_native::ABI_VERSION_V2 {
                return Err(format!("unsupported native ABI {}", descriptor.abi_version));
            }
            ExternalDescriptor::V2(unsafe { &*(descriptor as *const ject_native::PluginV2) })
        } else {
            let entry: libloading::Symbol<ject_native::EntryFn> = unsafe {
                library
                    .get(ject_native::ENTRY_SYMBOL)
                    .map_err(|e| format!("missing native ABI entry symbol: {e}"))?
            };
            let descriptor = unsafe { entry() }
                .as_ref()
                .ok_or_else(|| "plugin returned a null v1 descriptor".to_string())?;
            if descriptor.abi_version != ject_native::ABI_VERSION {
                return Err(format!("unsupported native ABI {}", descriptor.abi_version));
            }
            ExternalDescriptor::V1(unsafe { &*(descriptor as *const ject_native::PluginV1) })
        };
        let (abi_version, name_ptr, name_len, exports_ptr, exports_len) = match descriptor {
            ExternalDescriptor::V1(value) => (
                value.abi_version,
                value.name_ptr,
                value.name_len,
                value.exports_ptr,
                value.exports_len,
            ),
            ExternalDescriptor::V2(value) => (
                value.abi_version,
                value.name_ptr,
                value.name_len,
                value.exports_ptr,
                value.exports_len,
            ),
        };
        let name = abi_text(name_ptr, name_len, "plugin name")?;
        let export_text = abi_text(exports_ptr, exports_len, "plugin exports")?;
        let exports = export_text
            .lines()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();
        Ok(Self {
            name,
            abi_version,
            exports,
            descriptor,
            _library: library,
        })
    }
}

unsafe fn abi_text(ptr: *const u8, len: usize, what: &str) -> Result<String, String> {
    if ptr.is_null() && len != 0 {
        return Err(format!("{what} has an invalid buffer"));
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    };
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| format!("{what} is not UTF-8"))
}

impl NativeModule for DynamicNativeModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn exports(&self) -> HashMap<String, Value> {
        self.exports
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    Value::NativeFunction {
                        module: self.name.clone(),
                        name: name.clone(),
                    },
                )
            })
            .collect()
    }

    fn call(&self, function: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if !self.exports.iter().any(|name| name == function) {
            return Err(RuntimeError {
                message: format!("'{}' has no native export '{function}'", self.name),
            });
        }
        if matches!(self.descriptor, ExternalDescriptor::V2(_)) {
            CALLBACK_CONTEXTS.with(|contexts| {
                if let Some(context) = contexts.borrow_mut().last_mut() {
                    context.module = Some(self.name.clone());
                    context.descriptor = Some(self.descriptor);
                }
            });
        }
        let json_args: Result<Vec<_>, _> = args
            .iter()
            .map(|value| value_to_json(value, &self.name, Some(self.descriptor)))
            .collect();
        let encoded = serde_json::to_vec(&json_args.map_err(|message| RuntimeError { message })?)
            .map_err(|e| RuntimeError {
            message: format!("failed to encode native arguments: {e}"),
        })?;
        let result = unsafe {
            match self.descriptor {
                ExternalDescriptor::V1(descriptor) => (descriptor.call)(
                    function.as_ptr(),
                    function.len(),
                    encoded.as_ptr(),
                    encoded.len(),
                ),
                ExternalDescriptor::V2(descriptor) => (descriptor.call)(
                    function.as_ptr(),
                    function.len(),
                    encoded.as_ptr(),
                    encoded.len(),
                    &HOST_API,
                ),
            }
        };
        if result.ptr.is_null() && result.len != 0 {
            return Err(RuntimeError {
                message: "native plugin returned an invalid result buffer".to_string(),
            });
        }
        let bytes = if result.len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(result.ptr, result.len) }.to_vec()
        };
        unsafe {
            match self.descriptor {
                ExternalDescriptor::V1(descriptor) => (descriptor.free_buffer)(result),
                ExternalDescriptor::V2(descriptor) => (descriptor.free_buffer)(result),
            }
        };
        let envelope: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| RuntimeError {
                message: format!("native plugin returned invalid JSON: {e}"),
            })?;
        if let Some(error) = envelope.get("error").and_then(|v| v.as_str()) {
            return Err(RuntimeError {
                message: format!("{}::{function}: {error}", self.name),
            });
        }
        json_to_value(
            envelope
                .get("ok")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
            &self.name,
            self.descriptor,
        )
        .map_err(|message| RuntimeError { message })
    }
}

fn value_to_json(
    value: &Value,
    module: &str,
    descriptor: Option<ExternalDescriptor>,
) -> Result<serde_json::Value, String> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(value) => Ok((*value).into()),
        Value::Integer(value) => Ok((*value).into()),
        Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .or_else(|| ject_native::special_float(*value).ok())
            .ok_or_else(|| "native ABI cannot encode this floating-point value".to_string()),
        Value::String(value) => Ok(value.clone().into()),
        Value::Array(values) => values
            .borrow()
            .iter()
            .map(|value| value_to_json(value, module, descriptor))
            .collect::<Result<Vec<_>, _>>()
            .map(serde_json::Value::Array),
        Value::Dictionary(values) => values
            .borrow()
            .iter()
            .map(|(key, value)| Ok((key.clone(), value_to_json(value, module, descriptor)?)))
            .collect::<Result<serde_json::Map<_, _>, String>>()
            .map(serde_json::Value::Object),
        Value::Native(value) => {
            let resource = value.downcast_ref::<ExternalResource>().ok_or_else(|| {
                "this built-in native value cannot cross an external plugin ABI".to_string()
            })?;
            if resource.module != module {
                return Err(format!(
                    "resource belongs to native module '{}', not '{module}'",
                    resource.module
                ));
            }
            Ok(ject_native::resource(resource.id, &resource.type_name))
        }
        Value::Function { .. }
        | Value::ModuleFunction { .. }
        | Value::Lambda { .. }
        | Value::BuiltinFunction(_) => {
            if !matches!(descriptor, Some(ExternalDescriptor::V2(_))) {
                return Err("native ABI v1 cannot encode Ject callbacks".to_string());
            }
            CALLBACK_CONTEXTS.with(|contexts| {
                let mut contexts = contexts.borrow_mut();
                let context = contexts
                    .last_mut()
                    .ok_or_else(|| "no Ject callback context is active".to_string())?;
                context.callbacks.push(value.clone());
                Ok(ject_native::callback(context.callbacks.len() as u64))
            })
        }
        other => Err(format!(
            "native ABI v1 cannot encode Ject value of type '{}'",
            other.type_name()
        )),
    }
}

fn json_to_value(
    value: serde_json::Value,
    module: &str,
    descriptor: ExternalDescriptor,
) -> Result<Value, String> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(value) => Ok(Value::Bool(value)),
        serde_json::Value::Number(value) if value.is_i64() => {
            Ok(Value::Integer(value.as_i64().unwrap()))
        }
        serde_json::Value::Number(value) => value
            .as_f64()
            .map(Value::Float)
            .ok_or_else(|| "native plugin returned an unsupported number".to_string()),
        serde_json::Value::String(value) => Ok(Value::String(value)),
        serde_json::Value::Array(values) => values
            .into_iter()
            .map(|value| json_to_value(value, module, descriptor))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::array),
        serde_json::Value::Object(mut values) => {
            if let Some(special) = values.remove("$ject_float") {
                let special = special
                    .as_str()
                    .ok_or_else(|| "native special float tag must be a string".to_string())?;
                return match special {
                    "nan" => Ok(Value::Float(f64::NAN)),
                    "infinity" => Ok(Value::Float(f64::INFINITY)),
                    "negative_infinity" => Ok(Value::Float(f64::NEG_INFINITY)),
                    _ => Err(format!(
                        "native plugin returned unknown special float '{special}'"
                    )),
                };
            }
            if let Some(serde_json::Value::Object(mut resource)) = values.remove("$ject_resource") {
                let id = resource
                    .remove("id")
                    .and_then(|value| value.as_u64())
                    .ok_or_else(|| "native resource is missing an integer id".to_string())?;
                let type_name = resource
                    .remove("type")
                    .and_then(|value| value.as_str().map(str::to_string))
                    .ok_or_else(|| "native resource is missing a type name".to_string())?;
                return Ok(Value::Native(NativeValue::new(ExternalResource {
                    module: module.to_string(),
                    type_name,
                    id,
                    descriptor,
                })));
            }
            values
                .into_iter()
                .map(|(key, value)| Ok((key, json_to_value(value, module, descriptor)?)))
                .collect::<Result<HashMap<_, _>, String>>()
                .map(Value::dictionary)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn unused_call(
        _function_ptr: *const u8,
        _function_len: usize,
        _arguments_ptr: *const u8,
        _arguments_len: usize,
        _host: *const ject_native::HostV1,
    ) -> ject_native::Buffer {
        ject_native::Buffer::from_vec(b"{\"ok\":null}".to_vec())
    }

    unsafe extern "C" fn unused_drop(_id: u64) {}

    static TEST_PLUGIN: ject_native::PluginV2 = ject_native::PluginV2 {
        abi_version: ject_native::ABI_VERSION_V2,
        name_ptr: b"test".as_ptr(),
        name_len: 4,
        exports_ptr: std::ptr::null(),
        exports_len: 0,
        call: unused_call,
        free_buffer: ject_native::free_buffer,
        drop_resource: unused_drop,
    };

    #[test]
    fn callback_handles_invoke_ject_callables_and_return_values() {
        let mut interpreter = crate::interpreter::Interpreter::new();
        CALLBACK_CONTEXTS.with(|contexts| {
            contexts.borrow_mut().push(CallbackContext {
                interpreter: &mut interpreter,
                callbacks: Vec::new(),
                module: Some("test".to_string()),
                descriptor: Some(ExternalDescriptor::V2(&TEST_PLUGIN)),
            });
        });
        let encoded = value_to_json(
            &Value::BuiltinFunction("abs".to_string()),
            "test",
            Some(ExternalDescriptor::V2(&TEST_PLUGIN)),
        )
        .unwrap();
        assert_eq!(encoded, serde_json::json!({ "$ject_callback": 1 }));
        let arguments = serde_json::to_vec(&vec![serde_json::json!(-42)]).unwrap();
        assert_eq!(
            invoke_host_callback(1, arguments.as_ptr(), arguments.len()).unwrap(),
            serde_json::json!(42)
        );
        CALLBACK_CONTEXTS.with(|contexts| {
            contexts.borrow_mut().pop();
        });
    }

    #[test]
    fn native_wire_preserves_non_finite_floats() {
        let descriptor = ExternalDescriptor::V2(&TEST_PLUGIN);
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let encoded = value_to_json(&Value::Float(value), "test", Some(descriptor)).unwrap();
            let decoded = json_to_value(encoded, "test", descriptor).unwrap();
            let Value::Float(decoded) = decoded else {
                panic!("expected a float")
            };
            assert!(if value.is_nan() {
                decoded.is_nan()
            } else {
                decoded == value
            });
        }
    }
}
