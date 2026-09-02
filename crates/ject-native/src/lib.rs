//! Stable, minimal ABI shared by the Ject host and Rust native libraries.
//!
//! The ABI never passes Rust-owned types across the dynamic-library boundary.
//! Calls and results use UTF-8 JSON buffers in ABI v1. Later protocol versions
//! can add richer encodings without changing the descriptor entry symbol.

use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ABI_VERSION: u32 = 1;
pub const ENTRY_SYMBOL: &[u8] = b"ject_plugin_entry_v1\0";

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Buffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl Buffer {
    pub fn from_vec(mut bytes: Vec<u8>) -> Self {
        let result = Self {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
        };
        std::mem::forget(bytes);
        result
    }
}

pub type CallFn = unsafe extern "C" fn(
    function_ptr: *const u8,
    function_len: usize,
    arguments_ptr: *const u8,
    arguments_len: usize,
) -> Buffer;
pub type FreeBufferFn = unsafe extern "C" fn(Buffer);
pub type DropResourceFn = unsafe extern "C" fn(u64);

#[repr(C)]
#[derive(Debug)]
pub struct PluginV1 {
    pub abi_version: u32,
    pub name_ptr: *const u8,
    pub name_len: usize,
    /// UTF-8 exported function names separated by newlines.
    pub exports_ptr: *const u8,
    pub exports_len: usize,
    pub call: CallFn,
    pub free_buffer: FreeBufferFn,
    pub drop_resource: DropResourceFn,
}

// Descriptors only contain immutable static byte slices and function pointers.
unsafe impl Sync for PluginV1 {}

pub type EntryFn = unsafe extern "C" fn() -> *const PluginV1;
pub type Handler = fn(&str, Vec<serde_json::Value>) -> Result<serde_json::Value, String>;

/// Decode and dispatch one call from the native ABI.
///
/// # Safety
///
/// Each non-null pointer must reference a readable buffer of its paired length for
/// the duration of this call. Null pointers are only valid when their length is zero.
pub unsafe fn dispatch(
    handler: Handler,
    function_ptr: *const u8,
    function_len: usize,
    arguments_ptr: *const u8,
    arguments_len: usize,
) -> Buffer {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if (function_ptr.is_null() && function_len != 0)
            || (arguments_ptr.is_null() && arguments_len != 0)
        {
            return Err("host passed an invalid ABI buffer".to_string());
        }
        // SAFETY: pointers and lengths are validated by the host/plugin ABI contract.
        let function = unsafe { std::slice::from_raw_parts(function_ptr, function_len) };
        let arguments = unsafe { std::slice::from_raw_parts(arguments_ptr, arguments_len) };
        let function =
            std::str::from_utf8(function).map_err(|_| "function name is not UTF-8".to_string())?;
        let arguments: Vec<serde_json::Value> =
            serde_json::from_slice(arguments).map_err(|e| format!("invalid arguments: {e}"))?;
        handler(function, arguments)
    }));

    let envelope = match result {
        Ok(Ok(value)) => serde_json::json!({ "ok": value }),
        Ok(Err(error)) => serde_json::json!({ "error": error }),
        Err(_) => serde_json::json!({ "error": "native plugin panicked" }),
    };
    Buffer::from_vec(serde_json::to_vec(&envelope).expect("JSON envelope is serializable"))
}

pub fn drop_resource(handler: Handler, id: u64) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        handler("__drop_resource", vec![serde_json::json!(id)])
    }));
}

/// Constructs the wire representation of an opaque plugin-owned resource.
pub fn resource(id: u64, type_name: &str) -> serde_json::Value {
    serde_json::json!({ "$ject_resource": { "id": id, "type": type_name } })
}

/// Frees a buffer allocated by [`dispatch`].
///
/// # Safety
/// The buffer must have been returned by this SDK and must be freed exactly once.
pub unsafe extern "C" fn free_buffer(buffer: Buffer) {
    if !buffer.ptr.is_null() {
        // SAFETY: required by this function's contract.
        unsafe { drop(Vec::from_raw_parts(buffer.ptr, buffer.len, buffer.len)) };
    }
}

/// Defines the single stable entry point required by a Ject native library.
///
/// ```no_run
/// use serde_json::json;
///
/// fn call(name: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
///     match name { "double" => Ok(json!(args[0].as_i64().unwrap() * 2)), _ => Err("unknown".into()) }
/// }
/// ject_native::ject_plugin!("example", ["double"], call);
/// ```
#[macro_export]
macro_rules! ject_plugin {
    ($name:literal, [$($export:literal),* $(,)?], $handler:path) => {
        const __JECT_NAME: &[u8] = $name.as_bytes();
        const __JECT_EXPORTS: &[u8] = concat!($($export, "\n",)*).as_bytes();

        unsafe extern "C" fn __ject_call(
            function_ptr: *const u8,
            function_len: usize,
            arguments_ptr: *const u8,
            arguments_len: usize,
        ) -> $crate::Buffer {
            unsafe {
                $crate::dispatch($handler, function_ptr, function_len, arguments_ptr, arguments_len)
            }
        }

        unsafe extern "C" fn __ject_drop_resource(id: u64) {
            $crate::drop_resource($handler, id)
        }

        static __JECT_PLUGIN: $crate::PluginV1 = $crate::PluginV1 {
            abi_version: $crate::ABI_VERSION,
            name_ptr: __JECT_NAME.as_ptr(),
            name_len: __JECT_NAME.len(),
            exports_ptr: __JECT_EXPORTS.as_ptr(),
            exports_len: __JECT_EXPORTS.len(),
            call: __ject_call,
            free_buffer: $crate::free_buffer,
            drop_resource: __ject_drop_resource,
        };

        #[no_mangle]
        pub extern "C" fn ject_plugin_entry_v1() -> *const $crate::PluginV1 {
            &__JECT_PLUGIN
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicU64, Ordering};

    static DROPPED: AtomicU64 = AtomicU64::new(0);

    fn handler(function: &str, arguments: Vec<Value>) -> Result<Value, String> {
        match function {
            "add" => Ok(json!(
                arguments[0].as_i64().unwrap() + arguments[1].as_i64().unwrap()
            )),
            "fail" => Err("intentional failure".to_string()),
            "__drop_resource" => {
                DROPPED.store(arguments[0].as_u64().unwrap(), Ordering::SeqCst);
                Ok(Value::Null)
            }
            _ => Err("unknown function".to_string()),
        }
    }

    fn decode(buffer: Buffer) -> Value {
        let bytes = unsafe { std::slice::from_raw_parts(buffer.ptr, buffer.len) }.to_vec();
        unsafe { free_buffer(buffer) };
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn dispatches_success_and_error_envelopes() {
        let args = serde_json::to_vec(&vec![json!(20), json!(22)]).unwrap();
        let result = unsafe { dispatch(handler, b"add".as_ptr(), 3, args.as_ptr(), args.len()) };
        assert_eq!(decode(result), json!({ "ok": 42 }));

        let args = b"[]";
        let result = unsafe { dispatch(handler, b"fail".as_ptr(), 4, args.as_ptr(), args.len()) };
        assert_eq!(decode(result), json!({ "error": "intentional failure" }));
    }

    #[test]
    fn creates_and_drops_resource_wire_values() {
        assert_eq!(
            resource(7, "counter"),
            json!({ "$ject_resource": { "id": 7, "type": "counter" } })
        );
        drop_resource(handler, 7);
        assert_eq!(DROPPED.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn rejects_invalid_abi_input_without_panicking() {
        let result = unsafe { dispatch(handler, std::ptr::null(), 1, b"[]".as_ptr(), 2) };
        assert_eq!(
            decode(result),
            json!({ "error": "host passed an invalid ABI buffer" })
        );
    }
}
