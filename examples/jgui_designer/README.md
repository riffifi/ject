# JGUI Designer

Run `ject build` and `ject run` from this directory. Start with the editable Form,
switch to the Dashboard template, or create a blank document. Search the categorized
palette and add widgets at the document root or directly inside a selected layout.
Select widgets in the hierarchy to edit their properties, including comma-separated
selector choices. Move, duplicate, nest, unnest, or delete them from the inspector.
The center pane previews the production JGUI renderer. Window title, dimensions,
and the Linen, Midnight, egui-light, or egui-dark theme are editable at the top.

Save writes the interface document to the displayed path; Open restores it.
Undo and Redo keep up to 100 document edits in memory. Ctrl/Cmd+S, Ctrl/Cmd+O,
Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z, Ctrl/Cmd+D, and Delete provide normal editor controls.

Use the saved interface from Ject:

```ject
import "jgui" as gui
gui.show(parse_json(read_file("interface.json")))
```

The JSON document deliberately contains the serializable frontend. Keep callbacks
and application behavior in Ject source, where functions remain normal runtime
values instead of opaque strings embedded by a UI tool.
