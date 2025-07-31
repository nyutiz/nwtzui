use crate::Page;

#[derive(Clone, Default)]
pub struct Settings{
    //max_history_lines: usize,
}

impl Settings {
    pub fn ui(&mut self, ui: &mut egui::Ui, current_page: &mut Page) {
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("◀").clicked() {
                *current_page = Page::Terminal;
            }


            let rect = ui.max_rect();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "⚙ Settings",
                egui::FontId::proportional(18.0),
                ui.style().visuals.text_color(),
            );
        });


        ui.add_space(8.0);
        //ui.add(
        //    egui::Slider::new(&mut self.max_history_lines, 10..=500)
        //        .text("Max history lines"),
        //);


    }
}
