//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod frame;
mod terminal;
mod settings;
mod glob1env;

use eframe::{egui};
use std::process::exit;
use eframe::glow::Context;
use egui::{Id, Style, Visuals};
use egui::{Color32};
use crate::frame::custom_window_frame;
use crate::glob1env::Glob1Env;
use crate::settings::Settings;
use crate::terminal::Terminal;

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
            .with_resizable(false)

        ,..Default::default()
    };

    let mut dark_visuals = Visuals::dark();
    dark_visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
    dark_visuals.panel_fill = Color32::BLACK;

    let style = Style {
        visuals: dark_visuals,
        ..Default::default()
    };

    eframe::run_native(
        "nwtzui",
        options,
        Box::new(move |creation_context| {
            creation_context.egui_ctx.set_style(style.clone());
            Ok(Box::<NwtzUi>::default())
        }),
    )
}

#[derive(Clone, Default)]
enum Page {
    #[default] Terminal,
    Settings,
    Glob1Env,
}
#[derive(Clone)]
struct NwtzUi {
    title: String,
    current_page: Page,
    terminal: Terminal,
    settings: Settings,
    glob1env: Glob1Env,
    is_minimized: bool,
    normal_size: egui::Vec2,
    minimized_size: egui::Vec2,
    normal_pos: egui::Pos2,
    minimized_pos: egui::Pos2,
    position_initialized: bool,
    //pages: Vec<Page> Peut etre dans le futur pour la modulabilité
}

impl Default for NwtzUi {
    fn default() -> Self {
        Self {
            title: Default::default(),
            current_page: Default::default(),
            terminal: Default::default(),
            settings: Default::default(),
            glob1env: Default::default(),
            is_minimized: false,
            normal_size: egui::Vec2::new(400.0, 200.0),
            minimized_size: egui::Vec2::new(60.0, 60.0),
            normal_pos: egui::Pos2::new(0.0, 0.0),
            minimized_pos: egui::Pos2::new(0.0, 0.0),
            position_initialized: false,
        }
    }
}


impl eframe::App for NwtzUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.title = "nwtz".to_string();
        let title = self.title.clone();

        if !self.position_initialized {
            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                let window_size = self.normal_size;
                let pos = egui::pos2(
                    monitor_size.x - window_size.x - 10.0,
                    monitor_size.y - window_size.y - 55.0,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                self.position_initialized = true;
            }
        }
        
        if self.is_minimized {
            self.show_minimized_ui(ctx);
            return;
        }
        custom_window_frame(ctx, &title, |ui| {
            self.show_normal_ui(ui, frame);
        });

        if let Some(should_minimize) = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(Id::from("minimize_request"))) {
            if should_minimize {
                self.toggle_minimize(ctx);
                ctx.memory_mut(|mem| {
                    mem.data.insert_persisted::<bool>(Id::from("minimize_request"), false);
                });
            }
        }

        if let Some(next) = ctx.memory_mut(|mem| mem.data.get_persisted::<Option<Page>>(Id::from("next_page"))).flatten() {
            self.current_page = next;
            ctx.memory_mut(|mem| {
                mem.data.insert_persisted::<Option<Page>>(Id::from("next_page"), None);
            });
        }
    }
    

    fn on_exit(&mut self, _gl: Option<&Context>) {
        exit(0)
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

impl NwtzUi {
    fn toggle_minimize(&mut self, ctx: &egui::Context) {
        if !self.is_minimized {
            if let Some(outer) = ctx.input(|i| i.viewport().outer_rect) {
                self.normal_pos = outer.min;
            }
        }
        self.is_minimized = !self.is_minimized;
        if self.is_minimized {
            let size = self.minimized_size;
            let mut pos = self.minimized_pos;

            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                pos = egui::pos2(
                    monitor_size.x - size.x - 5.0,
                    monitor_size.y - size.y - 55.0,
                );
            }

            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        } else {
            let size = self.normal_size;
            let pos = self.normal_pos;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        }
    }

    fn show_minimized_ui(&mut self, ctx: &egui::Context) {
        use egui::{CentralPanel, Color32, Sense, Align2, PointerButton};

        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();

                let response = ui.interact(
                    rect,
                    ui.make_persistent_id("minimized_drag"),
                    Sense::click_and_drag(),
                );
                if response.drag_started_by(PointerButton::Primary) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                let center = rect.center();
                let radius = rect.width().min(rect.height()) / 2.0 - 5.0;
                ui.painter().circle_filled(center, radius, Color32::from_rgb(0, 0, 0));
                ui.painter().text(
                    center,
                    Align2::CENTER_CENTER,
                    "⚛",
                    egui::FontId::proportional(20.0),
                    Color32::WHITE,
                );

                let click_zone = ui.allocate_rect(rect, Sense::click());
                if click_zone.clicked() {
                    self.toggle_minimize(ctx);
                }
            });
    }

    fn show_normal_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

        match self.current_page {
            Page::Terminal => {
                let mut clone_ui = self.clone();
                self.terminal.ui(ui, &mut clone_ui);
            }
            Page::Settings => self.settings.ui(ui, &mut self.current_page),
            Page::Glob1Env => self.glob1env.ui(ui, &mut self.current_page),
        }
    }
}



