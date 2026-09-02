use crate::interpreter::RuntimeError;
use crate::value::Value;
use eframe::egui;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug)]
enum GuiWidget {
    Label(String),
    Separator,
    Button {
        id: String,
        text: String,
    },
    Input {
        id: String,
        label: String,
        initial: String,
    },
}

#[derive(Clone, Debug)]
struct GuiSpec {
    title: String,
    width: f32,
    height: f32,
    widgets: Vec<GuiWidget>,
}

#[derive(Default)]
struct GuiRegistry {
    specs: HashMap<i64, GuiSpec>,
}

#[derive(Default, Clone)]
struct GuiOutput {
    buttons: HashMap<String, bool>,
    inputs: HashMap<String, String>,
}

static NEXT_GUI_ID: AtomicI64 = AtomicI64::new(1);
static GUI_REGISTRY: OnceLock<Mutex<GuiRegistry>> = OnceLock::new();

#[derive(Clone, Debug)]
struct GuiHandle(i64);

impl crate::native::NativeObject for GuiHandle {
    fn type_name(&self) -> &'static str {
        "jgui_window"
    }

    fn display(&self) -> String {
        format!("<jgui window {}>", self.0)
    }

    fn native_eq(&self, other: &dyn crate::native::NativeObject) -> bool {
        other
            .as_any()
            .downcast_ref::<GuiHandle>()
            .is_some_and(|other| self.0 == other.0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn registry() -> &'static Mutex<GuiRegistry> {
    GUI_REGISTRY.get_or_init(|| Mutex::new(GuiRegistry::default()))
}

fn value_to_i64(value: &Value, what: &str) -> Result<i64, RuntimeError> {
    match value {
        Value::Native(value) => value
            .downcast_ref::<GuiHandle>()
            .map(|handle| handle.0)
            .ok_or_else(|| RuntimeError {
                message: format!("{} must be a JGUI window", what),
            }),
        _ => Err(RuntimeError {
            message: format!("{} must be a JGUI window", what),
        }),
    }
}

fn value_to_f32(value: &Value, what: &str) -> Result<f32, RuntimeError> {
    match value {
        Value::Integer(i) => Ok(*i as f32),
        Value::Float(f) => Ok(*f as f32),
        _ => Err(RuntimeError {
            message: format!("{} must be a number", what),
        }),
    }
}

fn value_to_string(value: &Value, what: &str) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(RuntimeError {
            message: format!("{} must be a string", what),
        }),
    }
}

struct RuntimeGuiApp {
    spec: GuiSpec,
    output: Arc<Mutex<GuiOutput>>,
}

/// Built-in native GUI module. Keeping its export table and dispatch here means
/// the interpreter does not need to know any GUI function names or prefixes.
pub struct JguiModule;

impl crate::native::NativeModule for JguiModule {
    fn name(&self) -> &'static str {
        "jgui"
    }

    fn exports(&self) -> HashMap<String, Value> {
        ["window", "label", "separator", "button", "input", "run"]
            .into_iter()
            .map(|name| {
                (
                    name.to_string(),
                    Value::NativeFunction {
                        module: "jgui".to_string(),
                        name: name.to_string(),
                    },
                )
            })
            .collect()
    }

    fn call(&self, fn_name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        call_jgui_function(fn_name, args)
    }
}

impl eframe::App for RuntimeGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading(&self.spec.title);
        ui.separator();

        if let Ok(mut out) = self.output.lock() {
            for widget in &self.spec.widgets {
                match widget {
                    GuiWidget::Label(text) => {
                        ui.label(text);
                    }
                    GuiWidget::Separator => {
                        ui.separator();
                    }
                    GuiWidget::Button { id, text } => {
                        let clicked = ui.button(text).clicked();
                        if clicked {
                            out.buttons.insert(id.clone(), true);
                        } else {
                            out.buttons.entry(id.clone()).or_insert(false);
                        }
                    }
                    GuiWidget::Input { id, label, initial } => {
                        let current = out
                            .inputs
                            .entry(id.clone())
                            .or_insert_with(|| initial.clone());
                        ui.horizontal(|ui| {
                            ui.label(label);
                            ui.text_edit_singleline(current);
                        });
                    }
                }
            }
        }
    }
}

pub fn call_jgui_function(name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match name {
        "window" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "window(title, width, height) requires 3 arguments".to_string(),
                });
            }
            let title = value_to_string(&args[0], "title")?;
            let width = value_to_f32(&args[1], "width")?;
            let height = value_to_f32(&args[2], "height")?;
            let id = NEXT_GUI_ID.fetch_add(1, Ordering::SeqCst);

            let spec = GuiSpec {
                title,
                width,
                height,
                widgets: Vec::new(),
            };

            let mut guard = registry().lock().map_err(|_| RuntimeError {
                message: "GUI registry is poisoned".to_string(),
            })?;
            guard.specs.insert(id, spec);
            Ok(Value::Native(crate::native::NativeValue::new(GuiHandle(
                id,
            ))))
        }
        "label" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "label(app, text) requires 2 arguments".to_string(),
                });
            }
            let id = value_to_i64(&args[0], "app")?;
            let text = value_to_string(&args[1], "text")?;
            let mut guard = registry().lock().map_err(|_| RuntimeError {
                message: "GUI registry is poisoned".to_string(),
            })?;
            let spec = guard.specs.get_mut(&id).ok_or_else(|| RuntimeError {
                message: format!("unknown GUI app handle: {}", id),
            })?;
            spec.widgets.push(GuiWidget::Label(text));
            Ok(Value::Nil)
        }
        "separator" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "separator(app) requires 1 argument".to_string(),
                });
            }
            let id = value_to_i64(&args[0], "app")?;
            let mut guard = registry().lock().map_err(|_| RuntimeError {
                message: "GUI registry is poisoned".to_string(),
            })?;
            let spec = guard.specs.get_mut(&id).ok_or_else(|| RuntimeError {
                message: format!("unknown GUI app handle: {}", id),
            })?;
            spec.widgets.push(GuiWidget::Separator);
            Ok(Value::Nil)
        }
        "button" => {
            if args.len() != 3 {
                return Err(RuntimeError {
                    message: "button(app, id, text) requires 3 arguments".to_string(),
                });
            }
            let app_id = value_to_i64(&args[0], "app")?;
            let id = value_to_string(&args[1], "id")?;
            let text = value_to_string(&args[2], "text")?;

            let mut guard = registry().lock().map_err(|_| RuntimeError {
                message: "GUI registry is poisoned".to_string(),
            })?;
            let spec = guard.specs.get_mut(&app_id).ok_or_else(|| RuntimeError {
                message: format!("unknown GUI app handle: {}", app_id),
            })?;
            spec.widgets.push(GuiWidget::Button { id, text });
            Ok(Value::Nil)
        }
        "input" => {
            if args.len() != 4 {
                return Err(RuntimeError {
                    message: "input(app, id, label, initial) requires 4 arguments".to_string(),
                });
            }
            let app_id = value_to_i64(&args[0], "app")?;
            let id = value_to_string(&args[1], "id")?;
            let label = value_to_string(&args[2], "label")?;
            let initial = value_to_string(&args[3], "initial")?;

            let mut guard = registry().lock().map_err(|_| RuntimeError {
                message: "GUI registry is poisoned".to_string(),
            })?;
            let spec = guard.specs.get_mut(&app_id).ok_or_else(|| RuntimeError {
                message: format!("unknown GUI app handle: {}", app_id),
            })?;
            spec.widgets.push(GuiWidget::Input { id, label, initial });
            Ok(Value::Nil)
        }
        "run" => {
            if args.len() != 1 {
                return Err(RuntimeError {
                    message: "run(app) requires 1 argument".to_string(),
                });
            }
            let app_id = value_to_i64(&args[0], "app")?;
            let spec = {
                let mut guard = registry().lock().map_err(|_| RuntimeError {
                    message: "GUI registry is poisoned".to_string(),
                })?;
                guard.specs.remove(&app_id).ok_or_else(|| RuntimeError {
                    message: format!("unknown GUI app handle: {}", app_id),
                })?
            };

            let output = Arc::new(Mutex::new(GuiOutput::default()));
            let output_for_app = Arc::clone(&output);
            let title = spec.title.clone();
            let viewport = egui::ViewportBuilder::default()
                .with_inner_size([spec.width, spec.height])
                .with_title(title.clone());
            let native_options = eframe::NativeOptions {
                viewport,
                ..Default::default()
            };

            eframe::run_native(
                &title,
                native_options,
                Box::new(move |_cc| {
                    Ok(Box::new(RuntimeGuiApp {
                        spec,
                        output: output_for_app,
                    }))
                }),
            )
            .map_err(|e| RuntimeError {
                message: format!("failed to run GUI app: {}", e),
            })?;

            let out = output.lock().map_err(|_| RuntimeError {
                message: "GUI output lock is poisoned".to_string(),
            })?;
            let mut buttons = HashMap::new();
            for (k, v) in &out.buttons {
                buttons.insert(k.clone(), Value::Bool(*v));
            }
            let mut inputs = HashMap::new();
            for (k, v) in &out.inputs {
                inputs.insert(k.clone(), Value::String(v.clone()));
            }

            let mut result = HashMap::new();
            result.insert("buttons".to_string(), Value::dictionary(buttons));
            result.insert("inputs".to_string(), Value::dictionary(inputs));
            Ok(Value::dictionary(result))
        }
        _ => Err(RuntimeError {
            message: format!("unknown GUI builtin: {}", name),
        }),
    }
}
