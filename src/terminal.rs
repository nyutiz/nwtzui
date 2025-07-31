use std::collections::VecDeque;
use std::path::Path;
use std::process::exit;
use eframe::epaint::Stroke;
use crate::{NwtzUi, Page, MAX_HISTORY_LINES, PROMPT};

#[derive(Clone)]
pub struct HistoryEntry {
    pub text: String,
    pub is_command: bool,
    pub action: Option<Page>,
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            is_command: false,
            action: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct Terminal{
    pub history: VecDeque<HistoryEntry>,
    pub current_input: String,
    pub scroll_to_bottom: bool,
}

impl Terminal {

    pub fn ui(&mut self, ui: &mut egui::Ui, nwtz_ui: &mut NwtzUi) {
        //let mut next_page = None;
        ui.visuals_mut().widgets.hovered = ui.visuals().widgets.inactive;

        let available_height = ui.available_height() - 45.0;
        if self.history.is_empty() {
            self.history.push_back(HistoryEntry {
                text: format!("Welcome to NWTZUI v{}", env!("CARGO_PKG_VERSION")).to_string(),
                is_command: false,
                action: None,
            });
            self.history.push_back(HistoryEntry {
                text: "Type 'help' for a list of commands".to_string(),
                is_command: false,
                action: None,
            });
            self.history.push_back(HistoryEntry {
                text: "".to_string(),
                is_command: false,
                action: None,
            });
        }

        egui::Frame::dark_canvas(ui.style())
            .fill(egui::Color32::from_rgb(0, 0, 0))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(available_height);

                scroll_area.show(ui, |ui| {

                    for entry in &self.history {
                        if let Some(action) = &entry.action {
                            let button = egui::Button::new(egui::RichText::new(&entry.text)
                                .size(12.0)
                                .color(egui::Color32::YELLOW))
                                .fill(egui::Color32::from_rgb(0, 0, 0));

                            if ui.add(button).clicked() {
                                ui.ctx().memory_mut(|mem| mem.data.insert_persisted::<Option<Page>>("next_page".into(), Option::from(action.clone())));
                            }
                        } else {
                            let text_color = if entry.is_command {
                                egui::Color32::CYAN
                            } else {
                                egui::Color32::from_rgb(255, 255, 255)
                            };

                            ui.colored_label(text_color, egui::RichText::new(&entry.text).size(14.0));
                        }
                    }
                    if self.scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.scroll_to_bottom = false;
                    }
                });
            });

        ui.add_space(4.0);

        ui.horizontal(|ui| {

            let mut s =String::from(PROMPT);
            s.truncate(s.len() - 1);
            
            ui.label(PROMPT);

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.current_input)
                    .desired_width(ui.available_width())
                    .hint_text("Type a command…")
                    .text_color(egui::Color32::CYAN)
                    .frame(false)
            );

            let rect = response.rect;
            let y_bottom = rect.max.y;
            let delta = 2.0;
            let y = y_bottom + delta;

            let p1 = egui::Pos2::new(rect.left()-2.0,  y);
            let p2 = egui::Pos2::new(rect.right()+2.0, y);
            ui.painter().line_segment([p1, p2], Stroke::new(1.0, egui::Color32::LIGHT_GRAY));

            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                if !self.current_input.is_empty() {
                    if let Some(page) = self.process_command(nwtz_ui.clone(), ui.ctx()) {
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_persisted::<Option<Page>>(
                                "next_page".into(), Some(page)
                            );
                        });
                    }
                }
            }

            response.request_focus();
        });
    }

    fn process_command(&mut self, nwtz_ui: NwtzUi, ctx: &egui::Context) -> Option<Page>{
        let command = self.current_input.clone();

        self.history.push_back(HistoryEntry {
            text: format!("{}{}", PROMPT, command),
            is_command: true,
            action: None,
        });
        let mut new_page: Option<Page> = None;

        let args:Vec<&str> = command.split(" ").collect();

        match command.as_str() {
            "help" => {
                self.add_response( "Available commands:");
                self.add_response( "  help  - Display this help message");
                self.add_response( "  clear - Clear terminal history");
                self.add_response( "  exit  - Exit the application");
                self.add_response( "  echo <text> - Echo text back to terminal");
                self.add_response( "  time  - Display current date and time");
                self.add_response( "  params  - Go to parameters");
                self.add_clickable("  ▶ Open Settings", Some(Page::Settings));
                self.add_response( "  env  - Go to environment");
                self.add_response( "    rd < path > - Read a file");
                //self.add_response( "    wr < path > < content > - Write to a file");
                self.add_clickable("  ▶ Open Environment", Some(Page::Glob1Env));
            },
            "clear" => {
                self.history.clear();
            },
            "exit" => {
                self.add_response("Goodbye!");
                exit(0)
            },
            "time" => {
                let now = chrono::Local::now();
                self.add_response(&format!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S")));
            },
            "params" => {
                new_page = Some(Page::Settings);
            },
            "open" => {
                println!("{:?}", args);
            },
            "minimize" => {
                self.add_response("Minimizing application...");
                ctx.memory_mut(|mem| {
                    mem.data.insert_persisted::<bool>("minimize_request".into(), true);
                });
            },
            "env" => {
                new_page = Some(Page::Glob1Env);
            },
            _ => {
                if command.starts_with("echo ") {
                    let echo_text = &command["echo ".len()..];
                    self.add_response(echo_text);
                }
                else if command.starts_with("env ") {
                    let env_command = &command["env ".len()..];
                    let args: Vec<&str> = env_command.split_whitespace().collect();

                    match args.get(0).map(|s| *s) {
                        Some("rd") if args.len() == 2 => {
                            let path = Path::new(args[1]);
                            let out = nwtz_ui.glob1env.rd(path).unwrap_or_else(|e| e);
                            self.add_response(&out);
                        }
                        //Some("wr") if args.len() >= 3 => {
                        //    let path = Path::new(args[1]);
                        //    let content = args[2..].join(" ");
//
                        //    let out = match nwtz_ui.glob1env.wr(path, content) {
                        //        Ok(_) => format!("Wrote to `{}`", path.display()),
                        //        Err(e) => format!("Error writing `{}`: {}", path.display(), e),
                        //    };
                        //    self.add_response(&out);
                        //}
                        _ => {
                            self.add_response("  env  - Go to environment");
                            self.add_response("    rd <path>            - Read a file");
                            //self.add_response("    wr <path> <content>  - Write to a file");
                        }
                    }
                }
                else {
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
        new_page
    }

    fn add_response(&mut self, text: &str) {
        self.history.push_back(HistoryEntry {
            text: text.to_string(),
            is_command: false,
            action: None,
        });
    }

    fn add_clickable(&mut self, text: &str, target_page: Option<Page>) {
        self.history.push_back(HistoryEntry {
            text: text.to_string(),
            is_command: false,
            action: target_page,
        });
    }
}