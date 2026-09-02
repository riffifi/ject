use eframe::egui;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Output {
    values: Map<String, Value>,
    clicked: HashMap<String, bool>,
    error: Option<String>,
}

struct DocumentApp {
    widgets: Vec<Value>,
    output: Arc<Mutex<Output>>,
    host: *const ject_native::HostV1,
}

fn text(widget: &Value, key: &str) -> String {
    widget
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn callback(widget: &Value, key: &str) -> Option<u64> {
    widget.get(key).and_then(ject_native::callback_id)
}

fn emit_event(
    host: *const ject_native::HostV1,
    callback: Option<u64>,
    kind: &str,
    id: &str,
    value: Value,
    values: &Map<String, Value>,
) -> Result<(), String> {
    let Some(callback) = callback else {
        return Ok(());
    };
    let event = json!({
        "kind": kind,
        "id": id,
        "value": value,
        "values": values,
    });
    unsafe { ject_native::invoke_callback(host, callback, vec![event]) }.map(|_| ())
}

impl eframe::App for DocumentApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Ok(mut output) = self.output.lock() else {
            return;
        };
        for widget in &self.widgets {
            match widget.get("kind").and_then(Value::as_str).unwrap_or("") {
                "heading" => {
                    ui.heading(text(widget, "text"));
                }
                "label" => {
                    ui.label(text(widget, "text"));
                }
                "separator" => {
                    ui.separator();
                }
                "spacer" => {
                    ui.add_space(widget.get("value").and_then(Value::as_f64).unwrap_or(8.0) as f32)
                }
                "progress" => {
                    let value = widget.get("value").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                    ui.add(
                        egui::ProgressBar::new(value.clamp(0.0, 1.0)).text(text(widget, "text")),
                    );
                }
                "text_input" | "multiline" => {
                    let id = text(widget, "id");
                    let label = text(widget, "label");
                    let initial = text(widget, "value");
                    let current = output
                        .values
                        .entry(id.clone())
                        .or_insert(Value::String(initial));
                    let mut value = current.as_str().unwrap_or_default().to_string();
                    ui.label(label);
                    let response = if widget["kind"] == "multiline" {
                        ui.add(egui::TextEdit::multiline(&mut value).desired_rows(5))
                    } else {
                        ui.text_edit_singleline(&mut value)
                    };
                    if response.changed() {
                        *current = Value::String(value);
                        let current = current.clone();
                        if let Err(error) = emit_event(
                            self.host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            current,
                            &output.values,
                        ) {
                            output.error = Some(error);
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
                "checkbox" => {
                    let id = text(widget, "id");
                    let initial = widget
                        .get("value")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let current = output
                        .values
                        .entry(id.clone())
                        .or_insert(Value::Bool(initial));
                    let mut checked = current.as_bool().unwrap_or(false);
                    if ui.checkbox(&mut checked, text(widget, "text")).changed() {
                        *current = Value::Bool(checked);
                        if let Err(error) = emit_event(
                            self.host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            Value::Bool(checked),
                            &output.values,
                        ) {
                            output.error = Some(error);
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
                "slider" => {
                    let id = text(widget, "id");
                    let initial = widget.get("value").and_then(Value::as_f64).unwrap_or(0.0);
                    let min = widget.get("min").and_then(Value::as_f64).unwrap_or(0.0);
                    let max = widget.get("max").and_then(Value::as_f64).unwrap_or(100.0);
                    let current = output.values.entry(id.clone()).or_insert(json!(initial));
                    let mut value = current.as_f64().unwrap_or(initial);
                    if ui
                        .add(egui::Slider::new(&mut value, min..=max).text(text(widget, "text")))
                        .changed()
                    {
                        *current = json!(value);
                        if let Err(error) = emit_event(
                            self.host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(value),
                            &output.values,
                        ) {
                            output.error = Some(error);
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
                "button" => {
                    let id = text(widget, "id");
                    if ui.button(text(widget, "text")).clicked() {
                        output.clicked.insert(id, true);
                        let id = text(widget, "id");
                        if let Err(error) = emit_event(
                            self.host,
                            callback(widget, "on_click"),
                            "click",
                            &id,
                            Value::Bool(true),
                            &output.values,
                        ) {
                            output.error = Some(error);
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if widget
                            .get("close")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                        {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
                _ => {
                    ui.colored_label(egui::Color32::RED, "Unknown JGUI widget");
                }
            }
        }
    }
}

fn call(
    function: &str,
    args: Vec<Value>,
    host: *const ject_native::HostV1,
) -> Result<Value, String> {
    if function != "run" {
        return Err(format!("unknown function '{function}'"));
    }
    let document = args
        .first()
        .and_then(Value::as_object)
        .ok_or("run expects a document")?;
    let title = document
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("JGUI")
        .to_string();
    let width = document
        .get("width")
        .and_then(Value::as_f64)
        .unwrap_or(680.0) as f32;
    let height = document
        .get("height")
        .and_then(Value::as_f64)
        .unwrap_or(560.0) as f32;
    let widgets = document
        .get("widgets")
        .and_then(Value::as_array)
        .cloned()
        .ok_or("document.widgets must be an array")?;
    let output = Arc::new(Mutex::new(Output::default()));
    let app_output = Arc::clone(&output);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([width, height]),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |_| {
            Ok(Box::new(DocumentApp {
                widgets,
                output: app_output,
                host,
            }))
        }),
    )
    .map_err(|e| format!("window failed: {e}"))?;
    let output = output.lock().map_err(|_| "JGUI output lock was poisoned")?;
    if let Some(error) = &output.error {
        return Err(format!("event callback failed: {error}"));
    }
    Ok(json!({ "values": output.values, "clicked": output.clicked }))
}

ject_native::ject_plugin_v2!("jgui", ["run"], call);
