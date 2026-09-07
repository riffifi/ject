use eframe::egui::{self, Color32, Stroke};

pub const NAMES: [&str; 4] = ["linen", "midnight", "egui-light", "egui-dark"];

pub fn validate(name: &str) -> Result<(), String> {
    if !NAMES.contains(&name) {
        return Err(format!(
            "unknown theme '{name}'; choose {}",
            NAMES.join(", ")
        ));
    }
    Ok(())
}

pub fn apply(ctx: &egui::Context, name: &str) -> Result<(), String> {
    validate(name)?;
    let dark = matches!(name, "midnight" | "egui-dark");
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    if !name.starts_with("egui-") {
        let (background, surface, foreground, accent) = if dark {
            (
                Color32::from_rgb(19, 25, 36),
                Color32::from_rgb(31, 41, 56),
                Color32::from_rgb(227, 234, 243),
                Color32::from_rgb(100, 195, 185),
            )
        } else {
            (
                Color32::from_rgb(245, 243, 237),
                Color32::from_rgb(255, 254, 250),
                Color32::from_rgb(38, 49, 47),
                Color32::from_rgb(24, 116, 102),
            )
        };
        visuals.panel_fill = background;
        visuals.window_fill = surface;
        visuals.extreme_bg_color = surface;
        visuals.override_text_color = Some(foreground);
        visuals.selection.bg_fill = accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
        visuals.hyperlink_color = accent;
        visuals.widgets.inactive.bg_fill = surface;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, accent);
        visuals.widgets.active.bg_fill = accent;
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = 7.into();
        }
    }
    ctx.set_visuals(visuals);
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.interact_size.y = 30.0;
    });
    Ok(())
}
