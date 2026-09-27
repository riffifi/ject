use eframe::egui;
mod theme;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
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
) -> Result<Value, String> {
    let Some(callback) = callback else {
        return Ok(Value::Null);
    };
    let event = json!({
        "kind": kind,
        "id": id,
        "value": value,
        "values": values,
    });
    unsafe { ject_native::invoke_callback(host, callback, vec![event]) }
}

fn merge_action(action: &Value, output: &mut Output) -> bool {
    let Some(action) = action.as_object() else {
        return false;
    };
    if let Some(values) = action
        .get("set")
        .or_else(|| action.get("values"))
        .and_then(Value::as_object)
    {
        output.values.extend(values.clone());
    }
    action
        .get("close")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn apply_action(action: Value, output: &mut Output, ui: &egui::Ui) {
    if merge_action(&action, output) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
    ui.ctx().request_repaint();
}

fn handle_event(
    ui: &egui::Ui,
    output: &mut Output,
    host: *const ject_native::HostV1,
    callback: Option<u64>,
    kind: &str,
    id: &str,
    value: Value,
) {
    match emit_event(host, callback, kind, id, value, &output.values) {
        Ok(action) => apply_action(action, output, ui),
        Err(error) => {
            output.error = Some(error);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn children(widget: &Value) -> &[Value] {
    widget
        .get("children")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn find_widget<'a>(widgets: &'a [Value], id: &str) -> Option<&'a Value> {
    for widget in widgets {
        if widget.get("id").and_then(Value::as_str) == Some(id) {
            return Some(widget);
        }
        if let Some(found) = find_widget(children(widget), id) {
            return Some(found);
        }
    }
    None
}

fn validate_document(document: &Map<String, Value>) -> Result<(), String> {
    const MAX_WIDGETS: usize = 10_000;
    const MAX_LAYOUT_DEPTH: usize = 64;

    fn validate_widgets(
        widgets: &[Value],
        path: &str,
        ids: &mut HashSet<String>,
        widget_count: &mut usize,
        depth: usize,
    ) -> Result<(), String> {
        const KINDS: [&str; 28] = [
            "row",
            "wrap",
            "column",
            "group",
            "card",
            "collapsible",
            "scroll",
            "grid",
            "heading",
            "label",
            "code",
            "link",
            "badge",
            "separator",
            "spacer",
            "progress",
            "meter",
            "value_text",
            "text_input",
            "password",
            "multiline",
            "checkbox",
            "toggle",
            "slider",
            "number_input",
            "select",
            "radio",
            "button",
        ];
        if depth > MAX_LAYOUT_DEPTH {
            return Err(format!(
                "{path} exceeds the maximum layout depth of {MAX_LAYOUT_DEPTH}"
            ));
        }
        for (index, widget) in widgets.iter().enumerate() {
            *widget_count = widget_count
                .checked_add(1)
                .ok_or("document contains too many widgets")?;
            if *widget_count > MAX_WIDGETS {
                return Err(format!("document contains more than {MAX_WIDGETS} widgets"));
            }
            let location = format!("{path}[{index}]");
            let kind = widget
                .get("kind")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{location}.kind must be a string"))?;
            if !KINDS.contains(&kind) {
                return Err(format!("{location} has unknown widget kind '{kind}'"));
            }
            let requires_id = matches!(
                kind,
                "meter"
                    | "value_text"
                    | "text_input"
                    | "password"
                    | "multiline"
                    | "checkbox"
                    | "toggle"
                    | "slider"
                    | "number_input"
                    | "select"
                    | "radio"
                    | "button"
            );
            if requires_id && widget.get("id").is_none() {
                return Err(format!("{location}.id must be a non-empty string"));
            }
            if let Some(id) = widget.get("id") {
                let id = id
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| format!("{location}.id must be a non-empty string"))?;
                if !ids.insert(id.to_string()) {
                    return Err(format!("{location}.id duplicates '{id}'"));
                }
            }
            for callback_key in ["on_change", "on_click"] {
                if let Some(value) = widget.get(callback_key) {
                    if !value.is_null() && ject_native::callback_id(value).is_none() {
                        return Err(format!(
                            "{location}.{callback_key} must be a callback or nil"
                        ));
                    }
                }
            }
            if let Some(children) = widget.get("children") {
                let children = children
                    .as_array()
                    .ok_or_else(|| format!("{location}.children must be an array"))?;
                validate_widgets(
                    children,
                    &format!("{location}.children"),
                    ids,
                    widget_count,
                    depth + 1,
                )?;
            }
        }
        Ok(())
    }

    let widgets = document
        .get("widgets")
        .and_then(Value::as_array)
        .ok_or("document.widgets must be an array")?;
    if document
        .get("title")
        .is_some_and(|title| !title.is_string())
    {
        return Err("document.title must be a string".into());
    }
    for key in ["width", "height"] {
        let value = document.get(key).and_then(Value::as_f64).unwrap_or(1.0);
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("document.{key} must be a positive number"));
        }
    }
    if document
        .get("state")
        .is_some_and(|state| !state.is_object())
    {
        return Err("document.state must be a dictionary".into());
    }
    theme::validate(
        document
            .get("theme")
            .and_then(Value::as_str)
            .unwrap_or("linen"),
    )?;
    let mut widget_count = 0;
    validate_widgets(
        widgets,
        "document.widgets",
        &mut HashSet::new(),
        &mut widget_count,
        0,
    )
}

fn render_widgets(
    ui: &mut egui::Ui,
    widgets: &[Value],
    output: &mut Output,
    host: *const ject_native::HostV1,
) {
    for widget in widgets {
        if widget.get("visible").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        let enabled = widget
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let response = ui.add_enabled_ui(enabled, |ui| {
            match widget.get("kind").and_then(Value::as_str).unwrap_or("") {
                "row" => {
                    ui.horizontal(|ui| render_widgets(ui, children(widget), output, host));
                }
                "wrap" => {
                    ui.horizontal_wrapped(|ui| render_widgets(ui, children(widget), output, host));
                }
                "column" => {
                    ui.vertical(|ui| render_widgets(ui, children(widget), output, host));
                }
                "group" => {
                    ui.group(|ui| {
                        let title = text(widget, "text");
                        if !title.is_empty() {
                            ui.strong(title);
                        }
                        render_widgets(ui, children(widget), output, host);
                    });
                }
                "card" => {
                    egui::Frame::group(ui.style())
                        .inner_margin(14.0)
                        .corner_radius(10.0)
                        .show(ui, |ui| render_widgets(ui, children(widget), output, host));
                }
                "collapsible" => {
                    egui::CollapsingHeader::new(text(widget, "text"))
                        .default_open(widget.get("open").and_then(Value::as_bool).unwrap_or(true))
                        .show(ui, |ui| render_widgets(ui, children(widget), output, host));
                }
                "scroll" => {
                    let height = widget
                        .get("height")
                        .and_then(Value::as_f64)
                        .unwrap_or(300.0) as f32;
                    egui::ScrollArea::both()
                        .max_height(height)
                        .show(ui, |ui| render_widgets(ui, children(widget), output, host));
                }
                "grid" => {
                    let columns = widget
                        .get("columns")
                        .and_then(Value::as_u64)
                        .unwrap_or(1)
                        .max(1) as usize;
                    let spacing =
                        widget.get("spacing").and_then(Value::as_f64).unwrap_or(8.0) as f32;
                    egui::Grid::new(widget as *const Value)
                        .num_columns(columns)
                        .spacing([spacing, spacing])
                        .show(ui, |ui| {
                            for (index, child) in children(widget).iter().enumerate() {
                                render_widgets(ui, std::slice::from_ref(child), output, host);
                                if (index + 1) % columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                }
                "heading" => {
                    ui.heading(text(widget, "text"));
                }
                "label" => {
                    ui.label(text(widget, "text"));
                }
                "code" => {
                    ui.label(egui::RichText::new(text(widget, "text")).monospace());
                }
                "link" => {
                    ui.hyperlink_to(text(widget, "text"), text(widget, "url"));
                }
                "badge" => {
                    let color = ui.visuals().selection.bg_fill;
                    ui.label(
                        egui::RichText::new(text(widget, "text"))
                            .background_color(color)
                            .color(egui::Color32::WHITE),
                    );
                }
                "separator" => {
                    ui.separator();
                }
                "spacer" => {
                    ui.add_space(widget.get("value").and_then(Value::as_f64).unwrap_or(8.0) as f32);
                }
                "progress" | "meter" => {
                    let id = text(widget, "id");
                    let fallback = widget.get("value").and_then(Value::as_f64).unwrap_or(0.0);
                    let value = output
                        .values
                        .get(&id)
                        .and_then(Value::as_f64)
                        .unwrap_or(fallback) as f32;
                    ui.add(
                        egui::ProgressBar::new(value.clamp(0.0, 1.0)).text(text(widget, "text")),
                    );
                }
                "value_text" => {
                    let id = text(widget, "id");
                    let value = output.values.get(&id).or_else(|| widget.get("value"));
                    let rendered = value
                        .map(|value| {
                            value
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| value.to_string())
                        })
                        .unwrap_or_else(|| "null".into());
                    ui.label(format!("{}{rendered}", text(widget, "text")));
                }
                "text_input" | "password" | "multiline" => {
                    let id = text(widget, "id");
                    let initial = text(widget, "value");
                    let mut value = output
                        .values
                        .get(&id)
                        .and_then(Value::as_str)
                        .unwrap_or(&initial)
                        .to_string();
                    ui.label(text(widget, "label"));
                    let response = if widget["kind"] == "multiline" {
                        ui.add(egui::TextEdit::multiline(&mut value).desired_rows(5))
                    } else if widget["kind"] == "password" {
                        ui.add(egui::TextEdit::singleline(&mut value).password(true))
                    } else {
                        ui.text_edit_singleline(&mut value)
                    };
                    output
                        .values
                        .entry(id.clone())
                        .or_insert_with(|| json!(initial));
                    if response.changed() {
                        output.values.insert(id.clone(), json!(value));
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(value),
                        );
                    }
                }
                "checkbox" | "toggle" => {
                    let id = text(widget, "id");
                    let initial = widget
                        .get("value")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let mut active = output
                        .values
                        .get(&id)
                        .and_then(Value::as_bool)
                        .unwrap_or(initial);
                    let changed = if widget["kind"] == "toggle" {
                        ui.toggle_value(&mut active, text(widget, "text")).changed()
                    } else {
                        ui.checkbox(&mut active, text(widget, "text")).changed()
                    };
                    output.values.entry(id.clone()).or_insert(json!(initial));
                    if changed {
                        output.values.insert(id.clone(), json!(active));
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(active),
                        );
                    }
                }
                "slider" | "number_input" => {
                    let id = text(widget, "id");
                    let initial = widget.get("value").and_then(Value::as_f64).unwrap_or(0.0);
                    let mut value = output
                        .values
                        .get(&id)
                        .and_then(Value::as_f64)
                        .unwrap_or(initial);
                    let response = if widget["kind"] == "number_input" {
                        ui.horizontal(|ui| {
                            ui.label(text(widget, "text"));
                            ui.add(
                                egui::DragValue::new(&mut value).speed(
                                    widget.get("speed").and_then(Value::as_f64).unwrap_or(1.0),
                                ),
                            )
                        })
                        .inner
                    } else {
                        let min = widget.get("min").and_then(Value::as_f64).unwrap_or(0.0);
                        let max = widget.get("max").and_then(Value::as_f64).unwrap_or(100.0);
                        ui.add(egui::Slider::new(&mut value, min..=max).text(text(widget, "text")))
                    };
                    output.values.entry(id.clone()).or_insert(json!(initial));
                    if response.changed() {
                        output.values.insert(id.clone(), json!(value));
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(value),
                        );
                    }
                }
                "select" => {
                    let id = text(widget, "id");
                    let initial = text(widget, "value");
                    let mut selected = output
                        .values
                        .get(&id)
                        .and_then(Value::as_str)
                        .unwrap_or(&initial)
                        .to_string();
                    let before = selected.clone();
                    egui::ComboBox::from_label(text(widget, "text"))
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            if let Some(options) = widget.get("options").and_then(Value::as_array) {
                                for option in options.iter().filter_map(Value::as_str) {
                                    ui.selectable_value(&mut selected, option.to_string(), option);
                                }
                            }
                        });
                    output.values.entry(id.clone()).or_insert(json!(initial));
                    if selected != before {
                        output.values.insert(id.clone(), json!(selected));
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(selected),
                        );
                    }
                }
                "radio" => {
                    let id = text(widget, "id");
                    let initial = text(widget, "value");
                    let mut selected = output
                        .values
                        .get(&id)
                        .and_then(Value::as_str)
                        .unwrap_or(&initial)
                        .to_string();
                    let before = selected.clone();
                    let title = text(widget, "text");
                    if !title.is_empty() {
                        ui.label(title);
                    }
                    if let Some(options) = widget.get("options").and_then(Value::as_array) {
                        for option in options.iter().filter_map(Value::as_str) {
                            ui.radio_value(&mut selected, option.to_string(), option);
                        }
                    }
                    output.values.entry(id.clone()).or_insert(json!(initial));
                    if selected != before {
                        output.values.insert(id.clone(), json!(selected));
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_change"),
                            "change",
                            &id,
                            json!(selected),
                        );
                    }
                }
                "button" => {
                    let id = text(widget, "id");
                    if ui.button(text(widget, "text")).clicked() {
                        output.clicked.insert(id.clone(), true);
                        handle_event(
                            ui,
                            output,
                            host,
                            callback(widget, "on_click"),
                            "click",
                            &id,
                            Value::Bool(true),
                        );
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
        });
        if let Some(tooltip) = widget.get("tooltip").and_then(Value::as_str) {
            response.response.on_hover_text(tooltip);
        }
    }
}

impl eframe::App for DocumentApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Ok(mut output) = self.output.lock() else {
            return;
        };
        render_widgets(ui, &self.widgets, &mut output, self.host);
    }
}

fn call(
    function: &str,
    args: Vec<Value>,
    host: *const ject_native::HostV1,
) -> Result<Value, String> {
    if function == "themes" {
        return Ok(json!(theme::NAMES));
    }
    if function == "validate" {
        let document = args
            .first()
            .and_then(Value::as_object)
            .ok_or("validate expects a document")?;
        validate_document(document)?;
        return Ok(Value::Bool(true));
    }
    if function == "dispatch" {
        let [Value::Object(document), Value::String(id), Value::String(kind), value] =
            args.as_slice()
        else {
            return Err("dispatch expects a document, widget ID, event kind, and value".into());
        };
        validate_document(document)?;
        let widgets = document
            .get("widgets")
            .and_then(Value::as_array)
            .ok_or("document.widgets must be an array")?;
        let widget = find_widget(widgets, id)
            .ok_or_else(|| format!("document has no widget with ID '{id}'"))?;
        let callback_key = match kind.as_str() {
            "change" => "on_change",
            "click" => "on_click",
            _ => return Err("event kind must be 'change' or 'click'".into()),
        };
        let callback = callback(widget, callback_key);
        let mut values = document
            .get("state")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if kind == "change" {
            values.insert(id.clone(), value.clone());
        }
        return emit_event(host, callback, kind, id, value.clone(), &values);
    }
    if function != "run" {
        return Err(format!("unknown function '{function}'"));
    }
    let document = args
        .first()
        .and_then(Value::as_object)
        .ok_or("run expects a document")?;
    validate_document(document)?;
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
    let state = document
        .get("state")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let output = Arc::new(Mutex::new(Output {
        values: state,
        ..Output::default()
    }));
    let app_output = Arc::clone(&output);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([width, height]),
        ..Default::default()
    };
    let theme_name = document
        .get("theme")
        .and_then(Value::as_str)
        .unwrap_or("linen")
        .to_string();
    theme::validate(&theme_name)?;
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            let _ = theme::apply(&cc.egui_ctx, &theme_name);
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

ject_native::ject_plugin_v2!("jgui", ["run", "themes", "validate", "dispatch"], call);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_actions_patch_state_and_request_close() {
        let mut output = Output::default();
        output.values.insert("tempo".into(), json!(120));
        let close = merge_action(
            &json!({ "set": { "tempo": 132, "status": "rendered" }, "close": true }),
            &mut output,
        );
        assert!(close);
        assert_eq!(output.values["tempo"], json!(132));
        assert_eq!(output.values["status"], json!("rendered"));
    }

    #[test]
    fn documents_are_validated_before_opening_a_window() {
        let valid =
            json!({"widgets":[{"kind":"column","children":[{"kind":"label"}]}],"theme":"linen"});
        assert!(validate_document(valid.as_object().unwrap()).is_ok());

        let unknown = json!({"widgets":[{"kind":"mystery"}]});
        assert_eq!(
            validate_document(unknown.as_object().unwrap()).unwrap_err(),
            "document.widgets[0] has unknown widget kind 'mystery'"
        );
        let bad_size = json!({"widgets":[],"width":0});
        assert_eq!(
            validate_document(bad_size.as_object().unwrap()).unwrap_err(),
            "document.width must be a positive number"
        );
        let duplicate =
            json!({"widgets":[{"kind":"button","id":"save"},{"kind":"button","id":"save"}]});
        assert_eq!(
            validate_document(duplicate.as_object().unwrap()).unwrap_err(),
            "document.widgets[1].id duplicates 'save'"
        );

        let mut nested = json!({"kind":"label"});
        for _ in 0..66 {
            nested = json!({"kind":"column","children":[nested]});
        }
        let deep = json!({"widgets":[nested]});
        assert!(validate_document(deep.as_object().unwrap())
            .unwrap_err()
            .contains("maximum layout depth"));
    }
}
