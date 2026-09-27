//! Blocking HTTP transport for the source-level `http` standard module.

use crate::interpreter::RuntimeError;
use crate::value::Value;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
static DOWNLOAD_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn invalid(message: impl Into<String>) -> RuntimeError {
    RuntimeError {
        message: message.into(),
    }
}

pub fn request(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let [Value::String(method), Value::String(url), Value::Dictionary(headers), body, Value::Integer(timeout_ms), Value::Integer(max_response_bytes)] =
        args.as_slice()
    else {
        return Err(invalid(
            "http.request() requires method, URL, header dictionary, body, timeout in milliseconds, and response limit in bytes",
        ));
    };
    if method.is_empty() || !method.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return Err(invalid(
            "http.request() method must contain only ASCII letters",
        ));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(invalid(
            "http.request() URL must start with http:// or https://",
        ));
    }
    let timeout_ms = u64::try_from(*timeout_ms)
        .ok()
        .filter(|timeout| *timeout > 0)
        .ok_or_else(|| invalid("http.request() timeout must be a positive integer"))?;
    let max_response_bytes = usize::try_from(*max_response_bytes)
        .ok()
        .filter(|limit| *limit <= MAX_RESPONSE_BYTES)
        .ok_or_else(|| {
            invalid("http.request() response limit must be from 0 to 16777216 bytes (16 MiB)")
        })?;
    let body = match body {
        Value::Nil => None,
        Value::String(text) => Some(text.as_bytes().to_vec()),
        Value::Array(values) => {
            let values = values.borrow();
            let mut bytes = Vec::with_capacity(values.len());
            for (index, value) in values.iter().enumerate() {
                let Value::Integer(number) = value else {
                    return Err(invalid(format!(
                        "http.request() body item {index} must be an integer from 0 to 255"
                    )));
                };
                let byte = u8::try_from(*number).map_err(|_| {
                    invalid(format!(
                        "http.request() body item {index} must be an integer from 0 to 255"
                    ))
                })?;
                bytes.push(byte);
            }
            Some(bytes)
        }
        _ => {
            return Err(invalid(
                "http.request() body must be a string, byte array, or nil",
            ));
        }
    };
    let mut request =
        ureq::request(&method.to_ascii_uppercase(), url).timeout(Duration::from_millis(timeout_ms));
    for (name, value) in headers.borrow().iter() {
        let Value::String(value) = value else {
            return Err(invalid(format!(
                "http.request() header '{name}' must have a string value"
            )));
        };
        request = request.set(name, value);
    }
    let response = match body {
        Some(bytes) => request.send_bytes(&bytes),
        None => request.call(),
    };
    let response = match response {
        Ok(response) | Err(ureq::Error::Status(_, response)) => response,
        Err(error) => {
            return Err(invalid(format!(
                "http.request() failed for '{url}': {error}"
            )));
        }
    };
    let status = response.status();
    let reason = response.status_text().to_string();
    let final_url = response.get_url().to_string();
    let mut response_headers = HashMap::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            response_headers.insert(name, Value::String(value.to_string()));
        }
    }
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(max_response_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| invalid(format!("http.request() failed to read '{url}': {error}")))?;
    if bytes.len() > max_response_bytes {
        return Err(invalid(format!(
            "http.request() response from '{url}' exceeds the {max_response_bytes} byte limit"
        )));
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(Value::dictionary(HashMap::from([
        ("status".to_string(), Value::Integer(i64::from(status))),
        ("reason".to_string(), Value::String(reason)),
        ("url".to_string(), Value::String(final_url)),
        (
            "success".to_string(),
            Value::Bool((200..300).contains(&status)),
        ),
        ("headers".to_string(), Value::dictionary(response_headers)),
        ("text".to_string(), Value::String(text)),
        (
            "bytes".to_string(),
            Value::array(
                bytes
                    .into_iter()
                    .map(|byte| Value::Integer(i64::from(byte)))
                    .collect(),
            ),
        ),
    ])))
}

fn temporary_download_path(destination: &Path) -> Result<PathBuf, RuntimeError> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| invalid("http.download() path must name a file"))?;

    for _ in 0..100 {
        let sequence = DOWNLOAD_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.ject-download-{}-{sequence}.tmp",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(invalid(format!(
        "http.download() could not create a temporary file beside '{}'",
        destination.display()
    )))
}

fn remove_temporary(path: &Path) {
    let _ = std::fs::remove_file(path);
}

pub fn download(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let [Value::String(url), Value::String(path), Value::Dictionary(headers), Value::Integer(timeout_ms), Value::Integer(max_bytes)] =
        args.as_slice()
    else {
        return Err(invalid(
            "http.download() requires URL, path, header dictionary, timeout in milliseconds, and byte limit",
        ));
    };
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(invalid(
            "http.download() URL must start with http:// or https://",
        ));
    }
    let destination = Path::new(path);
    if path.is_empty() || destination.file_name().is_none() {
        return Err(invalid("http.download() path must name a file"));
    }
    if destination.exists() {
        return Err(invalid(format!(
            "http.download() will not overwrite existing path '{}'",
            destination.display()
        )));
    }
    let timeout_ms = u64::try_from(*timeout_ms)
        .ok()
        .filter(|timeout| *timeout > 0)
        .ok_or_else(|| invalid("http.download() timeout must be a positive integer"))?;
    let max_bytes = u64::try_from(*max_bytes)
        .map_err(|_| invalid("http.download() byte limit must be zero or greater"))?;

    let mut request = ureq::get(url).timeout(Duration::from_millis(timeout_ms));
    for (name, value) in headers.borrow().iter() {
        let Value::String(value) = value else {
            return Err(invalid(format!(
                "http.download() header '{name}' must have a string value"
            )));
        };
        request = request.set(name, value);
    }
    let response = match request.call() {
        Ok(response) | Err(ureq::Error::Status(_, response)) => response,
        Err(error) => {
            return Err(invalid(format!(
                "http.download() failed for '{url}': {error}"
            )));
        }
    };
    let status = response.status();
    let reason = response.status_text().to_string();
    let final_url = response.get_url().to_string();
    let mut response_headers = HashMap::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            response_headers.insert(name, Value::String(value.to_string()));
        }
    }

    let temporary = temporary_download_path(destination)?;
    let result = (|| {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| {
                invalid(format!(
                    "http.download() could not create '{}': {error}",
                    temporary.display()
                ))
            })?;
        let mut reader = response.into_reader();
        let mut buffer = [0_u8; 64 * 1024];
        let mut bytes_written = 0_u64;
        loop {
            let count = reader.read(&mut buffer).map_err(|error| {
                invalid(format!("http.download() failed to read '{url}': {error}"))
            })?;
            if count == 0 {
                break;
            }
            let next_size = bytes_written
                .checked_add(count as u64)
                .ok_or_else(|| invalid("http.download() response size overflowed"))?;
            if next_size > max_bytes {
                return Err(invalid(format!(
                    "http.download() response from '{url}' exceeds the {max_bytes} byte limit"
                )));
            }
            output.write_all(&buffer[..count]).map_err(|error| {
                invalid(format!(
                    "http.download() could not write '{}': {error}",
                    temporary.display()
                ))
            })?;
            bytes_written = next_size;
        }
        output.sync_all().map_err(|error| {
            invalid(format!(
                "http.download() could not finish '{}': {error}",
                temporary.display()
            ))
        })?;
        drop(output);

        // A same-directory hard link atomically publishes the completed file and,
        // unlike rename on Unix, cannot replace a destination created meanwhile.
        std::fs::hard_link(&temporary, destination).map_err(|error| {
            invalid(format!(
                "http.download() could not publish '{}': {error}",
                destination.display()
            ))
        })?;
        remove_temporary(&temporary);

        Ok(Value::dictionary(HashMap::from([
            ("status".to_string(), Value::Integer(i64::from(status))),
            ("reason".to_string(), Value::String(reason)),
            ("url".to_string(), Value::String(final_url)),
            (
                "success".to_string(),
                Value::Bool((200..300).contains(&status)),
            ),
            ("headers".to_string(), Value::dictionary(response_headers)),
            ("path".to_string(), Value::String(path.clone())),
            (
                "bytes_written".to_string(),
                Value::Integer(bytes_written as i64),
            ),
        ])))
    })();
    if result.is_err() {
        remove_temporary(&temporary);
    }
    result
}
