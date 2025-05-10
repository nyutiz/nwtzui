#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui};
use std::collections::VecDeque;
use egui::ViewportCommand;

const MAX_HISTORY_LINES: usize = 100;
const PROMPT: &str = "> ";

// Commandes web sur un serveur
// Mise a jour depuis le github

// Commandes genre Open ...
// Ouvre pages webs
// Ajouter ... a ...  <- Comme pour task assistant

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size([400.0, 200.0])
            .with_max_inner_size([400.0, 200.0])
            .with_transparent(true)
            .with_always_on_top()

        ,..Default::default()
    };

    eframe::run_native(
        "egui Terminal",
        options,
        Box::new(|_cc| Ok(Box::<TerminalApp>::default())),
    )
}

struct HistoryEntry {
    text: String,
    is_command: bool,
}

#[derive(Default)]
struct TerminalApp {
    history: VecDeque<HistoryEntry>,
    current_input: String,
    scroll_to_bottom: bool,
}

impl eframe::App for TerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        custom_window_frame(ctx, "NWTZUI", |ui| {
            if self.history.is_empty() {
                self.history.push_back(HistoryEntry {
                    text: format!("Welcome to NWTZUI v{}", env!("CARGO_PKG_VERSION")).to_string(),
                    is_command: false,
                });
                self.history.push_back(HistoryEntry {
                    text: "Type 'help' for a list of commands".to_string(),
                    is_command: false,
                });
                self.history.push_back(HistoryEntry {
                    text: "".to_string(),
                    is_command: false,
                });
            }

            ctx.set_visuals(egui::Visuals::dark());

            let available_height = ui.available_height() - 45.0;

            egui::Frame::dark_canvas(ui.style())
                .fill(egui::Color32::from_rgb(10, 10, 10))
                .inner_margin(crate::egui::Margin::same(8))
                .show(ui, |ui| {
                    let scroll_area = egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .max_height(available_height);

                    scroll_area.show(ui, |ui| {
                        for entry in &self.history {
                            let text_color = if entry.is_command {
                                egui::Color32::from_rgb(0, 230, 0)
                            } else {
                                egui::Color32::from_rgb(200, 200, 200)
                            };


                            ui.colored_label(text_color, egui::RichText::new(&entry.text).size(14.0));
                        }
                    });
                });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(PROMPT);

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.current_input)
                        .desired_width(ui.available_width())
                        .hint_text("Type a command...")
                        .text_color(egui::Color32::from_rgb(0, 230, 0))
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) ||
                    ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.current_input.is_empty() {
                        self.process_command();
                    }
                }

                response.request_focus();

            });

        });
        //ctx.request_repaint();

    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

fn custom_window_frame(ctx: &egui::Context, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    use egui::{CentralPanel, UiBuilder};

    let panel_frame = egui::Frame::new()
        .fill(ctx.style().visuals.window_fill())
        .corner_radius(10)
        .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
        .outer_margin(1);

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 20.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title);

        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
            .shrink(4.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(ui: &mut egui::Ui, title_bar_rect: eframe::epaint::Rect, title: &str) {
    use egui::{Align2, FontId, Id, PointerButton, Sense, UiBuilder};

    let painter = ui.painter();

    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );


    if title_bar_response.double_clicked() {
        //ui.ctx().move right corner
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
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


fn close_maximize_minimize(ui: &mut egui::Ui) {
    use egui::{Button, RichText};

    let button_height = 16.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /*
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Restore window");
        if maximized_response.clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(false));
        }
    } else {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Maximize window");
        if maximized_response.clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
        }
    }
     */

    let minimized_response = ui
        .add(Button::new(RichText::new("➖").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}

impl TerminalApp {
    fn process_command(&mut self) {
        let command = self.current_input.clone();

        self.history.push_back(HistoryEntry {
            text: format!("{}{}", PROMPT, command),
            is_command: true,
        });

        let args:Vec<&str> = command.split(" ").collect();

        match args[0] {
            "help" => {
                self.add_response("Available commands:");
                self.add_response("  help  - Display this help message");
                self.add_response("  clear - Clear terminal history");
                self.add_response("  exit  - Exit the application");
                self.add_response("  echo <text> - Echo text back to terminal");
                self.add_response("  time  - Display current date and time");
            },
            "clear" => {
                self.history.clear();
            },
            "exit" => {
                self.add_response("Goodbye! (In a real app, this would exit)");
            },
            "time" => {
                let now = chrono::Local::now();
                self.add_response(&format!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S")));
            },
            "params" => {
                //OUVRIR PARAMS APP
            },
            "open" => {
                
                println!("{:?}", args);
                //OUVRIR PARAMS APP
            },
            "shutdown" => {
                //let _ = shutdown();
            },
            _ => {
                if command.starts_with("echo ") {
                    let echo_text = command[5..].trim();
                    self.add_response(echo_text);
                } else {
                    self.add_response(&format!("Unknown command: '{}'", command));
                    self.add_response("Type 'help' for a list of available commands");
                }
            }
        }

        self.current_input.clear();
        self.scroll_to_bottom = true;

        while self.history.len() > MAX_HISTORY_LINES {
            self.history.pop_front();
        }
    }

    fn add_response(&mut self, text: &str) {
        self.history.push_back(HistoryEntry {
            text: text.to_string(),
            is_command: false,
        });
    }
}