//! Blocking HTTP transport for the source-level `http` standard module.

use crate::interpreter::RuntimeError;
use crate::value::Value;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> RuntimeError {
    RuntimeError {
        message: message.into(),
    }
}

pub fn request(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let [Value::String(method), Value::String(url), Value::Dictionary(headers), body, Value::Integer(timeout_ms)] =
        args.as_slice()
    else {
        return Err(invalid(
            "http.request() requires method, URL, header dictionary, body, and timeout in milliseconds",
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
        .take(MAX_RESPONSE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| invalid(format!("http.request() failed to read '{url}': {error}")))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(invalid(format!(
            "http.request() response from '{url}' exceeds the 16 MiB limit"
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
