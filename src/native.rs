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
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

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

/// Loads a versioned `ject-native-1` dynamic library. The library owns its
/// descriptor and remains loaded for the rest of the process.
pub fn register_dynamic(path: &Path, expected_name: Option<&str>) -> Result<String, String> {
    let module = unsafe { DynamicNativeModule::load(path)? };
    let name = module.name.clone();
    if let Some(expected) = expected_name {
        if expected != name {
            return Err(format!(
                "native artifact declares module '{name}', expected '{expected}'"
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
    exports: Vec<String>,
    descriptor: &'static ject_native::PluginV1,
    _library: libloading::Library,
}

#[derive(Debug)]
struct ExternalResource {
    module: String,
    type_name: String,
    id: u64,
    descriptor: &'static ject_native::PluginV1,
}

impl Drop for ExternalResource {
    fn drop(&mut self) {
        unsafe { (self.descriptor.drop_resource)(self.id) };
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
        let entry: libloading::Symbol<ject_native::EntryFn> = unsafe {
            library
                .get(ject_native::ENTRY_SYMBOL)
                .map_err(|e| format!("missing ject_plugin_entry_v1: {e}"))?
        };
        let descriptor_ptr = unsafe { entry() };
        let descriptor = unsafe { descriptor_ptr.as_ref() }
            .ok_or_else(|| "plugin returned a null descriptor".to_string())?;
        if descriptor.abi_version != ject_native::ABI_VERSION {
            return Err(format!(
                "unsupported native ABI {}, host supports {}",
                descriptor.abi_version,
                ject_native::ABI_VERSION
            ));
        }
        let name = abi_text(descriptor.name_ptr, descriptor.name_len, "plugin name")?;
        let export_text = abi_text(
            descriptor.exports_ptr,
            descriptor.exports_len,
            "plugin exports",
        )?;
        let exports = export_text
            .lines()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();
        // The library is stored in the module and therefore outlives this reference.
        let descriptor = unsafe { &*(descriptor as *const ject_native::PluginV1) };
        Ok(Self {
            name,
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
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
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
        let json_args: Result<Vec<_>, _> = args
            .iter()
            .map(|value| value_to_json(value, &self.name))
            .collect();
        let encoded = serde_json::to_vec(&json_args.map_err(|message| RuntimeError { message })?)
            .map_err(|e| RuntimeError {
            message: format!("failed to encode native arguments: {e}"),
        })?;
        let result = unsafe {
            (self.descriptor.call)(
                function.as_ptr(),
                function.len(),
                encoded.as_ptr(),
                encoded.len(),
            )
        };
        if result.ptr.is_null() && result.len != 0 {
            return Err(RuntimeError {
                message: "native plugin returned an invalid result buffer".to_string(),
            });
        }
        let bytes = unsafe { std::slice::from_raw_parts(result.ptr, result.len) }.to_vec();
        unsafe { (self.descriptor.free_buffer)(result) };
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

fn value_to_json(value: &Value, module: &str) -> Result<serde_json::Value, String> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(value) => Ok((*value).into()),
        Value::Integer(value) => Ok((*value).into()),
        Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| "native ABI v1 cannot encode NaN or infinity".to_string()),
        Value::String(value) => Ok(value.clone().into()),
        Value::Array(values) => values
            .borrow()
            .iter()
            .map(|value| value_to_json(value, module))
            .collect::<Result<Vec<_>, _>>()
            .map(serde_json::Value::Array),
        Value::Dictionary(values) => values
            .borrow()
            .iter()
            .map(|(key, value)| Ok((key.clone(), value_to_json(value, module)?)))
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
        other => Err(format!(
            "native ABI v1 cannot encode Ject value of type '{}'",
            other.type_name()
        )),
    }
}

fn json_to_value(
    value: serde_json::Value,
    module: &str,
    descriptor: &'static ject_native::PluginV1,
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
