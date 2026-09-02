//! Stable, minimal ABI shared by the Ject host and Rust native libraries.
//!
//! The ABI never passes Rust-owned types across the dynamic-library boundary.
//! Calls and results use UTF-8 JSON buffers in ABI v1. Later protocol versions
//! can add richer encodings without changing the descriptor entry symbol.

use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ABI_VERSION_V1: u32 = 1;
pub const ABI_VERSION: u32 = ABI_VERSION_V1;
pub const ABI_VERSION_V2: u32 = 2;
pub const ENTRY_SYMBOL: &[u8] = b"ject_plugin_entry_v1\0";
pub const ENTRY_SYMBOL_V2: &[u8] = b"ject_plugin_entry_v2\0";

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
pub type HostCallFn = unsafe extern "C" fn(u64, *const u8, usize) -> Buffer;

#[repr(C)]
pub struct HostV1 {
    pub call_callback: HostCallFn,
    pub free_buffer: FreeBufferFn,
}

pub type CallFnV2 = unsafe extern "C" fn(
    function_ptr: *const u8,
    function_len: usize,
    arguments_ptr: *const u8,
    arguments_len: usize,
    host: *const HostV1,
) -> Buffer;

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

#[repr(C)]
#[derive(Debug)]
pub struct PluginV2 {
    pub abi_version: u32,
    pub name_ptr: *const u8,
    pub name_len: usize,
    pub exports_ptr: *const u8,
    pub exports_len: usize,
    pub call: CallFnV2,
    pub free_buffer: FreeBufferFn,
    pub drop_resource: DropResourceFn,
}

// Descriptors only contain immutable static byte slices and function pointers.
unsafe impl Sync for PluginV1 {}
unsafe impl Sync for PluginV2 {}

pub type EntryFn = unsafe extern "C" fn() -> *const PluginV1;
pub type EntryFnV2 = unsafe extern "C" fn() -> *const PluginV2;
pub type Handler = fn(&str, Vec<serde_json::Value>) -> Result<serde_json::Value, String>;
pub type HandlerV2 =
    fn(&str, Vec<serde_json::Value>, *const HostV1) -> Result<serde_json::Value, String>;

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
        let function = if function_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(function_ptr, function_len) }
        };
        let arguments = if arguments_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(arguments_ptr, arguments_len) }
        };
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

/// Decode and dispatch an ABI v2 call with access to the host callback table.
///
/// # Safety
/// Buffer pointers follow the same contract as [`dispatch`]. `host` must either be
/// null or point to a valid [`HostV1`] for the duration of the call.
pub unsafe fn dispatch_v2(
    handler: HandlerV2,
    function_ptr: *const u8,
    function_len: usize,
    arguments_ptr: *const u8,
    arguments_len: usize,
    host: *const HostV1,
) -> Buffer {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if (function_ptr.is_null() && function_len != 0)
            || (arguments_ptr.is_null() && arguments_len != 0)
        {
            return Err("host passed an invalid ABI buffer".to_string());
        }
        let function = if function_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(function_ptr, function_len) }
        };
        let arguments = if arguments_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(arguments_ptr, arguments_len) }
        };
        let function =
            std::str::from_utf8(function).map_err(|_| "function name is not UTF-8".to_string())?;
        let arguments = serde_json::from_slice(arguments)
            .map_err(|error| format!("invalid arguments: {error}"))?;
        handler(function, arguments, host)
    }));
    encode_envelope(result)
}

fn encode_envelope(
    result: Result<Result<serde_json::Value, String>, Box<dyn std::any::Any + Send>>,
) -> Buffer {
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

pub fn drop_resource_v2(handler: HandlerV2, id: u64) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        handler(
            "__drop_resource",
            vec![serde_json::json!(id)],
            std::ptr::null(),
        )
    }));
}

/// Constructs the wire representation of an opaque plugin-owned resource.
pub fn resource(id: u64, type_name: &str) -> serde_json::Value {
    serde_json::json!({ "$ject_resource": { "id": id, "type": type_name } })
}

pub fn callback(id: u64) -> serde_json::Value {
    serde_json::json!({ "$ject_callback": id })
}

pub fn callback_id(value: &serde_json::Value) -> Option<u64> {
    value
        .get("$ject_callback")
        .and_then(serde_json::Value::as_u64)
}

/// Invokes a Ject callable previously passed to the plugin.
///
/// # Safety
/// `host` must be the host table supplied to the current ABI v2 plugin call and
/// must only be used for that call's duration and on the calling thread.
pub unsafe fn invoke_callback(
    host: *const HostV1,
    id: u64,
    arguments: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let host =
        unsafe { host.as_ref() }.ok_or_else(|| "host callback API is unavailable".to_string())?;
    let encoded = serde_json::to_vec(&arguments)
        .map_err(|error| format!("failed to encode callback arguments: {error}"))?;
    let result = unsafe { (host.call_callback)(id, encoded.as_ptr(), encoded.len()) };
    if result.ptr.is_null() && result.len != 0 {
        return Err("host returned an invalid callback buffer".to_string());
    }
    let bytes = if result.len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(result.ptr, result.len) }.to_vec()
    };
    unsafe { (host.free_buffer)(result) };
    let envelope: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("host returned invalid callback JSON: {error}"))?;
    if let Some(error) = envelope.get("error").and_then(serde_json::Value::as_str) {
        return Err(error.to_string());
    }
    Ok(envelope
        .get("ok")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
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

/// Defines an ABI v2 plugin that can invoke Ject callbacks through [`invoke_callback`].
#[macro_export]
macro_rules! ject_plugin_v2 {
    ($name:literal, [$($export:literal),* $(,)?], $handler:path) => {
        const __JECT_NAME_V2: &[u8] = $name.as_bytes();
        const __JECT_EXPORTS_V2: &[u8] = concat!($($export, "\n",)*).as_bytes();

        unsafe extern "C" fn __ject_call_v2(
            function_ptr: *const u8,
            function_len: usize,
            arguments_ptr: *const u8,
            arguments_len: usize,
            host: *const $crate::HostV1,
        ) -> $crate::Buffer {
            unsafe {
                $crate::dispatch_v2(
                    $handler,
                    function_ptr,
                    function_len,
                    arguments_ptr,
                    arguments_len,
                    host,
                )
            }
        }

        unsafe extern "C" fn __ject_drop_resource_v2(id: u64) {
            $crate::drop_resource_v2($handler, id);
        }

        static __JECT_PLUGIN_V2: $crate::PluginV2 = $crate::PluginV2 {
            abi_version: $crate::ABI_VERSION_V2,
            name_ptr: __JECT_NAME_V2.as_ptr(),
            name_len: __JECT_NAME_V2.len(),
            exports_ptr: __JECT_EXPORTS_V2.as_ptr(),
            exports_len: __JECT_EXPORTS_V2.len(),
            call: __ject_call_v2,
            free_buffer: $crate::free_buffer,
            drop_resource: __ject_drop_resource_v2,
        };

        #[no_mangle]
        pub extern "C" fn ject_plugin_entry_v2() -> *const $crate::PluginV2 {
            &__JECT_PLUGIN_V2
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

    unsafe extern "C" fn echo_callback(
        id: u64,
        arguments_ptr: *const u8,
        arguments_len: usize,
    ) -> Buffer {
        let arguments: Vec<Value> = serde_json::from_slice(unsafe {
            std::slice::from_raw_parts(arguments_ptr, arguments_len)
        })
        .unwrap();
        Buffer::from_vec(
            serde_json::to_vec(&json!({ "ok": { "id": id, "arguments": arguments } })).unwrap(),
        )
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

    #[test]
    fn invokes_callbacks_through_the_host_table() {
        let host = HostV1 {
            call_callback: echo_callback,
            free_buffer,
        };
        assert_eq!(
            unsafe { invoke_callback(&host, 7, vec![json!("event")]) }.unwrap(),
            json!({ "id": 7, "arguments": ["event"] })
        );
        assert_eq!(callback(7), json!({ "$ject_callback": 7 }));
        assert_eq!(callback_id(&callback(7)), Some(7));
    }
}
