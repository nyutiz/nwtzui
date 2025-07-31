use eframe::epaint::Color32;
use egui::ViewportCommand;
use egui::{CentralPanel, UiBuilder};

pub fn custom_window_frame(ctx: &egui::Context, title: &str, add_contents: impl FnOnce(&mut egui::Ui), ) {
    let panel_frame = egui::Frame::new()
        .fill(Color32::from_rgb(0, 0, 0))
        .corner_radius(10.0)
        .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
        .outer_margin(1.0);

    CentralPanel::default()
        .frame(panel_frame)
        .show(ctx, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_height = 20.0;

            let title_bar_rect = {
                let mut r = app_rect;
                r.max.y = r.min.y + title_bar_height;
                r
            };
            title_bar_ui(ui, title_bar_rect, title);

            let content_rect = {
                let mut r = app_rect;
                r.min.y = title_bar_rect.max.y;
                r
            }
                .shrink(4.0);

            let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
            add_contents(&mut content_ui);
        });
}

pub fn title_bar_ui(ui: &mut egui::Ui, title_bar_rect: eframe::epaint::Rect, title: &str, ) {
    use egui::{Align2, FontId, Id, PointerButton, Sense, UiBuilder};

    let response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );
    ui.painter().text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    if response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui);
        },
    );
}

pub fn close_maximize_minimize(ui: &mut egui::Ui) {
    use egui::{Button, RichText};

    let size = 16.0;
    if ui.add(Button::new(RichText::new("❌").size(size))).on_hover_text("Close").clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
    }
    if ui.add(Button::new(RichText::new("➖").size(size))).on_hover_text("Minimize").clicked() {
        ui.ctx().memory_mut(|mem| {
            mem.data.insert_persisted::<bool>("minimize_request".into(), true);
        });
    }
}
