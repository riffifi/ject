use super::{egui, json, render_widgets, theme, Output, Value};

struct Designer {
    document: Value,
    selected: Vec<usize>,
    path: String,
    status: String,
    preview: Output,
    undo: Vec<Value>,
    redo: Vec<Value>,
    next_id: usize,
    catalog: Vec<Value>,
}

fn nodes<'a>(document: &'a Value, parent: &[usize]) -> &'a [Value] {
    let mut nodes = document["widgets"].as_array().unwrap();
    for index in parent {
        nodes = nodes[*index]["children"].as_array().unwrap();
    }
    nodes
}

fn nodes_mut<'a>(document: &'a mut Value, parent: &[usize]) -> &'a mut Vec<Value> {
    let mut nodes = document["widgets"].as_array_mut().unwrap();
    for index in parent {
        nodes = nodes[*index]["children"].as_array_mut().unwrap();
    }
    nodes
}

fn is_container(node: &Value) -> bool {
    node.get("children").is_some_and(Value::is_array)
}

fn selected_container(document: &Value, selected: &[usize]) -> Option<Vec<usize>> {
    let (&index, parent) = selected.split_last()?;
    nodes(document, parent)
        .get(index)
        .filter(|node| is_container(node))?;
    Some(selected.to_vec())
}

fn make_widget(definition: &Value, id: usize) -> Result<Value, String> {
    let kind = definition["kind"]
        .as_str()
        .ok_or("component kind must be a string")?;
    let mut widget = definition["defaults"]
        .as_object()
        .cloned()
        .ok_or_else(|| format!("component '{kind}' defaults must be a dictionary"))?;
    widget.insert("kind".into(), json!(kind));
    if widget.contains_key("id") {
        widget.insert("id".into(), json!(format!("{kind}_{id}")));
    }
    Ok(Value::Object(widget))
}

fn tree(ui: &mut egui::Ui, nodes: &[Value], path: &mut Vec<usize>, selected: &mut Vec<usize>) {
    for (i, node) in nodes.iter().enumerate() {
        path.push(i);
        let kind = node["kind"].as_str().unwrap_or("?");
        let detail = node["text"]
            .as_str()
            .or_else(|| node["label"].as_str())
            .unwrap_or("");
        let label = if detail.is_empty() {
            kind.to_string()
        } else {
            format!("{kind}  {detail}")
        };
        if ui.selectable_label(*selected == *path, label).clicked() {
            *selected = path.clone();
        }
        if let Some(children) = node["children"].as_array() {
            ui.indent(format!("tree{path:?}"), |ui| {
                tree(ui, children, path, selected)
            });
        }
        path.pop();
    }
}

fn property_editor(ui: &mut egui::Ui, node: &mut Value) {
    for (key, value) in node.as_object_mut().unwrap() {
        if key == "children" || value.is_null() {
            continue;
        }
        ui.label(key);
        match value {
            Value::String(text) => {
                ui.text_edit_singleline(text);
            }
            Value::Bool(enabled) => {
                ui.checkbox(enabled, if *enabled { "true" } else { "false" });
            }
            Value::Number(number) => {
                let mut numeric = number.as_f64().unwrap_or(0.0);
                if ui.add(egui::DragValue::new(&mut numeric)).changed() {
                    *value = json!(numeric);
                }
            }
            Value::Array(items) if key == "options" => {
                let mut text = items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ");
                if ui.text_edit_singleline(&mut text).changed() {
                    *items = text
                        .split(',')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(|item| json!(item))
                        .collect();
                }
                ui.small("Comma-separated choices");
            }
            _ => {
                ui.weak("Not editable");
            }
        }
    }
}

impl Designer {
    fn add_widget(&mut self, definition: &Value, inside_selected: bool) {
        let parent = if inside_selected {
            selected_container(&self.document, &self.selected).unwrap_or_default()
        } else {
            Vec::new()
        };
        let Ok(node) = make_widget(definition, self.next_id) else {
            self.status = "The component catalog contains an invalid entry.".into();
            return;
        };
        self.next_id += 1;
        let siblings = nodes_mut(&mut self.document, &parent);
        let index = siblings.len();
        siblings.push(node);
        self.selected = parent;
        self.selected.push(index);
    }

    fn move_selected(&mut self, offset: isize) {
        let Some((&index, parent)) = self.selected.split_last() else {
            return;
        };
        let siblings = nodes_mut(&mut self.document, parent);
        let target = index as isize + offset;
        if target >= 0 && target < siblings.len() as isize {
            siblings.swap(index, target as usize);
            *self.selected.last_mut().unwrap() = target as usize;
        }
    }

    fn duplicate_selected(&mut self) {
        let Some((&index, parent)) = self.selected.split_last() else {
            return;
        };
        let siblings = nodes_mut(&mut self.document, parent);
        if let Some(node) = siblings.get(index).cloned() {
            siblings.insert(index + 1, node);
            *self.selected.last_mut().unwrap() += 1;
        }
    }

    fn delete_selected(&mut self) {
        let Some((&index, parent)) = self.selected.split_last() else {
            return;
        };
        nodes_mut(&mut self.document, parent).remove(index);
        self.selected.clear();
    }

    fn nest_selected(&mut self) {
        let Some((&index, parent)) = self.selected.split_last() else {
            return;
        };
        if index == 0 || !is_container(&nodes(&self.document, parent)[index - 1]) {
            self.status = "The preceding widget is not a layout container.".into();
            return;
        }
        let siblings = nodes_mut(&mut self.document, parent);
        let node = siblings.remove(index);
        siblings[index - 1]["children"]
            .as_array_mut()
            .unwrap()
            .push(node);
        self.selected.clear();
    }

    fn unnest_selected(&mut self) {
        if self.selected.len() < 2 {
            self.status = "The selected widget is already at the document root.".into();
            return;
        }
        let child_index = *self.selected.last().unwrap();
        let parent_path = &self.selected[..self.selected.len() - 1];
        let parent_index = *parent_path.last().unwrap();
        let grandparent = &parent_path[..parent_path.len() - 1];
        let node = nodes_mut(&mut self.document, parent_path).remove(child_index);
        nodes_mut(&mut self.document, grandparent).insert(parent_index + 1, node);
        self.selected = grandparent.to_vec();
        self.selected.push(parent_index + 1);
    }
}

impl eframe::App for Designer {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let before = self.document.clone();
        let mut history_action = false;
        ui.heading("JGUI Designer");
        ui.horizontal(|ui| {
            ui.label("Document");
            ui.text_edit_singleline(&mut self.path);
            if ui.button("Save").clicked() {
                self.status = match std::fs::write(
                    &self.path,
                    serde_json::to_string_pretty(&self.document).unwrap(),
                ) {
                    Ok(()) => "Saved".into(),
                    Err(error) => error.to_string(),
                };
            }
            if ui.button("Open").clicked() {
                match load(&self.path, &self.catalog) {
                    Ok(document) => {
                        self.document = document;
                        self.selected.clear();
                        self.preview = Output::default();
                        self.status = "Opened".into();
                    }
                    Err(error) => self.status = error,
                }
            }
            if ui.button("Undo").clicked() {
                history_action = true;
                if let Some(document) = self.undo.pop() {
                    self.redo.push(self.document.clone());
                    self.document = document;
                    self.selected.clear();
                }
            }
            if ui.button("Redo").clicked() {
                history_action = true;
                if let Some(document) = self.redo.pop() {
                    self.undo.push(self.document.clone());
                    self.document = document;
                    self.selected.clear();
                }
            }
        });

        let mut title = self.document["title"]
            .as_str()
            .unwrap_or("JGUI")
            .to_string();
        let mut width = self.document["width"].as_f64().unwrap_or(800.0);
        let mut height = self.document["height"].as_f64().unwrap_or(600.0);
        let mut theme_name = self.document["theme"]
            .as_str()
            .unwrap_or("linen")
            .to_string();
        ui.horizontal(|ui| {
            ui.label("Window");
            if ui.text_edit_singleline(&mut title).changed() {
                self.document["title"] = json!(title);
            }
            ui.label("Width");
            if ui
                .add(egui::DragValue::new(&mut width).range(240.0..=3840.0))
                .changed()
            {
                self.document["width"] = json!(width);
            }
            ui.label("Height");
            if ui
                .add(egui::DragValue::new(&mut height).range(160.0..=2160.0))
                .changed()
            {
                self.document["height"] = json!(height);
            }
            egui::ComboBox::from_id_salt("theme")
                .selected_text(&theme_name)
                .show_ui(ui, |ui| {
                    for name in theme::NAMES {
                        ui.selectable_value(&mut theme_name, name.into(), name);
                    }
                });
            ui.label(&self.status);
        });
        self.document["theme"] = json!(theme_name);
        let _ = theme::apply(ui.ctx(), &theme_name);
        ui.separator();

        ui.columns(3, |columns| {
            columns[0].strong("Widgets & hierarchy");
            let can_add_inside = selected_container(&self.document, &self.selected).is_some();
            columns[0].weak(if can_add_inside {
                "Add to root, or inside the selected layout"
            } else {
                "Select a layout to add inside it"
            });
            egui::ScrollArea::vertical()
                .id_salt("palette")
                .max_height(220.0)
                .show(&mut columns[0], |ui| {
                    for definition in self.catalog.clone() {
                        let kind = definition["kind"].as_str().unwrap_or("invalid");
                        ui.horizontal(|ui| {
                            if ui.button(format!("+ {kind}")).clicked() {
                                self.add_widget(&definition, false);
                            }
                            if can_add_inside && ui.small_button("inside").clicked() {
                                self.add_widget(&definition, true);
                            }
                        });
                    }
                });
            columns[0].separator();
            egui::ScrollArea::vertical()
                .id_salt("hierarchy")
                .show(&mut columns[0], |ui| {
                    tree(
                        ui,
                        self.document["widgets"].as_array().unwrap(),
                        &mut Vec::new(),
                        &mut self.selected,
                    );
                });

            columns[1].strong("Live preview");
            egui::ScrollArea::both()
                .id_salt("preview")
                .show(&mut columns[1], |ui| {
                    render_widgets(
                        ui,
                        self.document["widgets"].as_array().unwrap(),
                        &mut self.preview,
                        std::ptr::null(),
                    );
                });

            columns[2].strong("Properties");
            if let Some((&index, parent)) = self.selected.split_last() {
                let siblings = nodes_mut(&mut self.document, parent);
                if let Some(node) = siblings.get_mut(index) {
                    property_editor(&mut columns[2], node);
                }
                columns[2].separator();
                columns[2].horizontal_wrapped(|ui| {
                    if ui.button("Up").clicked() {
                        self.move_selected(-1);
                    }
                    if ui.button("Down").clicked() {
                        self.move_selected(1);
                    }
                    if ui.button("Duplicate").clicked() {
                        self.duplicate_selected();
                    }
                    if ui.button("Nest").clicked() {
                        self.nest_selected();
                    }
                    if ui.button("Unnest").clicked() {
                        self.unnest_selected();
                    }
                    if ui.button("Delete").clicked() {
                        self.delete_selected();
                    }
                });
            } else {
                columns[2].label("Select a widget in the hierarchy to edit it.");
            }
        });

        if before != self.document && !history_action {
            self.redo.clear();
            self.undo.push(before);
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
        }
    }
}

fn validate_catalog(catalog: &[Value]) -> Result<(), String> {
    if catalog.is_empty()
        || catalog.iter().any(|definition| {
            !definition["kind"].is_string() || !definition["defaults"].is_object()
        })
    {
        return Err("Designer component catalog requires kind and defaults".into());
    }
    Ok(())
}

fn validate_document(document: &Value, catalog: &[Value]) -> Result<(), String> {
    fn validate_nodes(nodes: &Value, catalog: &[Value]) -> bool {
        nodes.as_array().is_some_and(|nodes| {
            nodes.iter().all(|node| {
                node.is_object()
                    && node["kind"]
                        .as_str()
                        .is_some_and(|kind| catalog.iter().any(|item| item["kind"] == kind))
                    && (node.get("children").is_none()
                        || validate_nodes(&node["children"], catalog))
            })
        })
    }
    if !document.is_object() || !validate_nodes(&document["widgets"], catalog) {
        return Err("Document requires a widgets array with valid nested widgets".into());
    }
    theme::validate(document["theme"].as_str().unwrap_or("linen"))?;
    Ok(())
}

fn load(path: &str, catalog: &[Value]) -> Result<Value, String> {
    let document: Value =
        serde_json::from_str(&std::fs::read_to_string(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    validate_document(&document, catalog)?;
    Ok(document)
}

pub fn run(path: &str, catalog: &[Value]) -> Result<Value, String> {
    validate_catalog(catalog)?;
    let document = if std::path::Path::new(path).exists() {
        load(path, catalog)?
    } else {
        json!({"title":"My application","theme":"linen","width":800,"height":600,"state":{},"widgets":[]})
    };
    let designer = Designer {
        document,
        selected: vec![],
        path: path.into(),
        status: "Build the interface, preview it, then save.".into(),
        preview: Output::default(),
        undo: vec![],
        redo: vec![],
        next_id: 1,
        catalog: catalog.to_vec(),
    };
    eframe::run_native(
        "JGUI Designer",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let _ = theme::apply(&cc.egui_ctx, "linen");
            Ok(Box::new(designer))
        }),
    )
    .map_err(|error| error.to_string())?;
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_nested_designer_documents() {
        let catalog = json!([
            {"kind":"column","defaults":{"children":[]}},
            {"kind":"label","defaults":{"text":"Label"}}
        ]);
        let catalog = catalog.as_array().unwrap();
        let document = json!({"theme":"linen","widgets":[{"kind":"column","children":[{"kind":"label","text":"Hi"}]}]});
        assert!(validate_document(&document, catalog).is_ok());
        assert!(validate_document(&json!({"widgets":[{"text":"missing kind"}]}), catalog).is_err());
        assert!(validate_document(&json!({"theme":"nope","widgets":[]}), catalog).is_err());
    }

    #[test]
    fn inserts_into_selected_containers() {
        let mut designer = Designer {
            document: json!({"widgets":[{"kind":"column","children":[]}]}),
            selected: vec![0],
            path: String::new(),
            status: String::new(),
            preview: Output::default(),
            undo: vec![],
            redo: vec![],
            next_id: 1,
            catalog: vec![],
        };
        designer.add_widget(
            &json!({"kind":"button","defaults":{"id":"","text":"Button"}}),
            true,
        );
        assert_eq!(
            designer.document["widgets"][0]["children"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(designer.selected, vec![0, 0]);
    }
}
