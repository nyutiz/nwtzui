use std::collections::HashMap;
#[allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use eframe::epaint::Color32;
use egui::{CentralPanel, OutputCommand, RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use egui_inbox::UiInbox;
use nwtzlang::{match_arg_to_string, mk_fn, mk_null, mk_object};
use nwtzlang::environment::Environment;
use nwtzlang::evaluator::evaluate;
use nwtzlang::lexer::tokenize;
use nwtzlang::parser::Parser;
use nwtzlang::runtime::RuntimeVal;
use nwtzlang::types::ValueType::{NativeFn, Object};
use tokio::runtime::Runtime;
use crate::Page;
#[derive(Debug, Clone)]
pub struct Glob1Env {
    pub current_path: PathBuf,
    pub root_content: Vec<FsEntry>,
    pub lang_env: Environment,
    pub inbox: Arc<UiInbox<String>>,
    pub execution_started: bool,
    pub message_buffer: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum FsEntry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug)]
pub struct File {
    pub name: String,
    pub content: String,
    pub system: bool,
}

#[derive(Clone, Debug)]
pub struct Directory {
    pub name: String,
    pub content: Vec<FsEntry>,
    pub system: bool,
}

impl Default for Glob1Env {
    fn default() -> Self {
        init_system()
    }
}

impl Glob1Env {

    pub fn ui(&mut self, ui: &mut egui::Ui, current_page: &mut Page){
        //let ui_percent = ui.available_width();

        let file_id = ui.make_persistent_id("glob1env_selected_file");
        let mut frame_side = egui::Frame::side_top_panel(ui.style());
        frame_side.outer_margin.right = 8;

        egui::SidePanel::left("glob1env_explorer").exact_width(ui.available_width()/100.0*37.0).resizable(false).frame(frame_side).show_inside(ui, |ui|{
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("<").size(14.0))).clicked() {
                    //println!("CP {:?}", self.current_path);
                    if self.current_path.parent().is_none() {
                        *current_page = Page::Terminal;
                    } else {
                        self.current_path.pop();
                    }

                }

                ui.heading(self.current_path.to_string_lossy());

            });
            ui.separator();

            ScrollArea::vertical().auto_shrink(false).max_height(ui.available_height() - 1.5 * ui.spacing().interact_size.y - ui.spacing().item_spacing.y, ).show(ui, |ui| {
                match self.ls() {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                FsEntry::Directory(dir) => {
                                    if ui.selectable_label(false, RichText::new(format!("📁 {}", dir.name)).color(if dir.system { Color32::CYAN } else { Color32::WHITE }).size(14.0), ).clicked() {
                                        self.push(&dir.name);
                                    }
                                }
                                FsEntry::File(file) => {
                                    if ui.selectable_label(false, RichText::new(format!("📃 {}", file.name)).color(if file.system { Color32::CYAN } else { Color32::WHITE }).size(14.0)).clicked() {
                                        ui.ctx().data_mut(|d| {
                                            d.insert_temp(file_id, file);
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        ui.label("Impossible de lister le répertoire.");
                        eprintln!("{}", e);
                    }
                }
            });
        });

        let mut frame_right = egui::Frame::side_top_panel(ui.style());
        frame_right.outer_margin.left = 8;

        let mut frame_center = egui::Frame::central_panel(ui.style());
        frame_center.outer_margin.left = 8;
        frame_center.outer_margin.right = 8;

        CentralPanel::default().frame(frame_center).show_inside(ui, |ui| {
            if let Some(file) = ui.ctx().data(|d| d.get_temp::<File>(file_id)) {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(&file.name).size(14.0).color(Color32::CYAN));
                    });
                });

                ui.separator();

                ScrollArea::vertical().auto_shrink(false).scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                    
                    let content: Vec<String> = file.content.clone().split('\n').map(String::from).collect();
                    
                    if file.name.ends_with(".pwd"){
                        for content in content {
                            match content.as_str() { 
                                ctn if ctn.starts_with("[PWD]") => {
                                    let parts: Vec<&str> = ctn[5..].splitn(2, "::").collect();
                                    let service = parts.get(0).map(|s| s.trim()).unwrap_or("");
                                    let secret  = parts.get(1).map(|s| s.trim()).unwrap_or("");
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{service} :")).size(14.0));
                                        if ui.button(RichText::new("copy").size(14.0)).clicked() {
                                            ui.output_mut(|o| {
                                                o.commands.push(OutputCommand::CopyText(secret.to_string()));
                                            });
                                        };
                                    });
                                },
                                //ctn if ctn.starts_with("[]") => {

                                //},
                                ctn => {
                                    ui.label(RichText::new(ctn).size(14.0));
                                },
                            }
                        }
                    }
                    else if file.name.ends_with(".nwtz!") {
                        let file_name = file.name.clone();
                        let env_clone = self.clone();
                        let mut lang_env = self.lang_env.clone();
                        let sender = self.inbox.sender();

                        // Seulement démarrer l'exécution si elle n'est pas déjà en cours
                        if !self.execution_started {
                            self.execution_started = true;

                            let thread_sender = sender.clone();

                            std::thread::spawn(move || {
                                let rt = Runtime::new().unwrap();
                                rt.block_on(async move {
                                    let thread_sender_clone = thread_sender.clone();

                                    let execution_result = tokio::task::spawn_blocking(move || {
                                        let log_sender = thread_sender_clone.clone();
                                        lang_env.set_var(
                                            "log".to_string(),
                                            mk_fn(Arc::new(move |args, _| {
                                                for arg in args {
                                                    let s = match_arg_to_string(&*arg);
                                                    let _ = log_sender.send(s);
                                                }
                                                mk_null()
                                            })),
                                            Some(NativeFn),
                                        );

                                        let log_sender_button = thread_sender_clone.clone();
                                        lang_env.set_var(
                                            "button".to_string(),
                                            mk_fn(Arc::new(move |args, _| {
                                                for arg in args {
                                                    let text = match_arg_to_string(&*arg);
                                                    let _ = log_sender_button.send(format!("[b] {}", text));
                                                }
                                                mk_null()
                                            })),
                                            Some(NativeFn),
                                        );

                                        let log_sender_ui = thread_sender_clone.clone();
                                        lang_env.set_var(
                                            "ui".to_string(),
                                            mk_object({
                                                let mut props: HashMap<String, Box<dyn RuntimeVal + Send + Sync>> = HashMap::new();

                                                let button_sender = log_sender_ui.clone();
                                                let pwd_sender = log_sender_ui.clone();

                                                props.insert("button".to_string(), mk_fn(Arc::new(move |args, _scope| {
                                                    let text = match_arg_to_string(&*args[0]);
                                                    let _ = button_sender.send(format!("[BTN] {}", text));
                                                    mk_null()
                                                })));

                                                props.insert("password".to_string(), mk_fn(Arc::new(move |args, _scope| {
                                                    if args.len() == 2 {
                                                        let service = match_arg_to_string(&*args[0]);
                                                        let secret = match_arg_to_string(&*args[1]);
                                                        let _ = pwd_sender.send(format!("[PWD] {service}::{secret}"));
                                                    }
                                                    mk_null()
                                                })));

                                                props
                                            }),
                                            Some(Object),
                                        );

                                        let rt = tokio::runtime::Handle::current();
                                        let file_content = rt.block_on(async {
                                            env_clone.rd(&*PathBuf::from(&file_name))
                                        });

                                        match file_content {
                                            Ok(content) => {
                                                let tokens = tokenize(content);
                                                let mut parser = Parser::new(tokens);

                                                let h: HashMap<String, String> = HashMap::new();
                                                //h.insert("system".to_string(), env_clone.get_content("system.nwtz").unwrap_or_default());
                                                //h.insert("copy".to_string(), env_clone.get_content("copy.nwtz").unwrap_or_default());
                                                //h.insert("paste".to_string(), env_clone.get_content("paste.nwtz").unwrap_or_default());
                                                //h.insert("cut".to_string(), env_clone.get_content("cut.nwtz").unwrap_or_default());
                                                //h.insert("start".to_string(), env_clone.get_content("start.nwtz").unwrap_or_default());
                                                //h.insert("stop".to_string(), env_clone.get_content("stop.nwtz").unwrap_or_default());
                                                //h.insert("switch".to_string(), env_clone.get_content("switch.nwtz").unwrap_or_default());

                                                parser.provide_import(h);
                                                let ast = parser.produce_ast();
                                                evaluate(Box::new(ast), &mut lang_env)
                                            }
                                            Err(e) => {
                                                let error_sender = thread_sender_clone.clone();
                                                let _ = error_sender.send(format!("Erreur chargement {}: {}", file_name, e));
                                                mk_null()
                                            }
                                        }
                                    }).await;

                                    if let Err(e) = execution_result {
                                        let error_sender = thread_sender.clone();
                                        let _ = error_sender.send(format!("Execution failed: {}", e));
                                    }
                                });
                            });
                        }

                        // Collecter les nouveaux messages et les ajouter au buffer
                        for msg in self.inbox.as_ref().read(ui) {
                            if !self.message_buffer.contains(&msg) {
                                self.message_buffer.push(msg);
                            }
                        }

                        // Afficher tous les messages du buffer persistant
                        for msg in &self.message_buffer {
                            if let Some(s) = msg.strip_prefix("[PWD] ") {
                                let parts: Vec<&str> = s.splitn(2, "::").collect();
                                let service = parts.get(0).map(|s| s.trim()).unwrap_or("");
                                let secret  = parts.get(1).map(|s| s.trim()).unwrap_or("");

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{service} :")).size(14.0));
                                    if ui.button(RichText::new("copy").size(14.0)).clicked() {
                                        ui.output_mut(|o| {
                                            o.commands.push(OutputCommand::CopyText(secret.to_string()));
                                        });
                                    };
                                });
                            } else if let Some(s) = msg.strip_prefix("[BTN] ") {
                                ui.colored_label(Color32::LIGHT_BLUE, s);
                            } else {
                                ui.label(msg);
                            }
                        }
                    }                    else {
                        for c in content.iter() {
                            ui.label(RichText::new(c).size(14.0));
                        }
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a file to view its contents");
                });
            }
        });
    }

    pub fn push(&mut self, segment: &str) {
        let mut s = self.current_path.to_string_lossy().to_string();
        if !s.ends_with('/') {
            s.push('/');
        }
        s.push_str(segment);
        self.current_path = PathBuf::from(s);
    }

    #[warn(dead_code)]
    pub fn join_fn(path: &Path, segment: &str) -> PathBuf {
        let mut s = path.to_string_lossy().to_string();
        if !s.ends_with('/') {
            s.push('/');
        }
        s.push_str(segment);
        PathBuf::from(s)
    }

    pub fn find_directory_mut<'a>(entries: &'a mut [FsEntry], path_parts: &[&str], ) -> Option<&'a mut Directory> {
        if path_parts.is_empty() {

            return None;
        }

        let current_part = path_parts[0];
        let remaining_parts = &path_parts[1..];

        for entry in entries.iter_mut() {
            if let FsEntry::Directory(dir) = entry {
                if dir.name == current_part {
                    return if remaining_parts.is_empty() {
                        Some(dir)
                    } else {
                        Self::find_directory_mut(&mut dir.content, remaining_parts)
                    }
                }
            }
        }

        None
    }

    pub fn add_entry_to_path(&mut self, path: &Path, entry: FsEntry) -> Result<(), String> {
        if !path.has_root() {
            return Err("Le chemin doit être absolu (commencer par '/')".to_string());
        }

        if path == Path::new("/") {
            //if let FsEntry::File(ref file) = entry {
            //    if file.executable {
            //        self.apps.push(file.clone());
            //    }
            //}
            self.root_content.push(entry);
            return Ok(());
        }

        let components: Vec<&str> = path.to_str().unwrap_or("")
            .split('/')
            .filter(|&s| !s.is_empty())
            .collect();

        if let Some(target_dir) = Self::find_directory_mut(&mut self.root_content, &components) {
            //if let FsEntry::File(ref file) = entry {
            //    if file.executable {
            //        self.apps.push(file.clone());
            //    }
            //}
            target_dir.content.push(entry);
            Ok(())
        } else {
            Err(format!("Le chemin '{}' n'a pas été trouvé.", path.display()))
        }
    }

    pub fn ls(&self) -> Result<Vec<FsEntry>, String> {
        let mut res = Vec::new();

        let components = split_components(&self.current_path);


        if components.is_empty() {
            res.extend(self.root_content.iter().cloned());
            return Ok(res);
        }

        let mut cloned = self.root_content.clone();
        if let Some(dir) = Self::find_directory_mut(&mut cloned, &components) {
            res.extend(dir.content.iter().cloned());
            return Ok(res);
        }

        Err(format!(
            "Répertoire introuvable : '{}'",
            self.current_path.display()
        ))
    }

    #[warn(dead_code)]
    pub fn ls_path(&self, path: &Path) -> Result<Vec<FsEntry>, String> {
        let mut res = Vec::new();

        let components = split_components(path);

        if components.is_empty() {
            res.extend(self.root_content.iter().cloned());
            return Ok(res);
        }

        let mut cloned = self.root_content.clone();
        if let Some(dir) = Self::find_directory_mut(&mut cloned, &components) {
            res.extend(dir.content.iter().cloned());
            return Ok(res);
        }

        Err(format!(
            "Répertoire introuvable : '{}'",
            path.display()
        ))
    }

    #[warn(dead_code)]
    pub fn cd(&mut self, path: &Path) -> &PathBuf {
        //println!("Changement de répertoire vers '{}'", path.display());
        self.current_path = PathBuf::from(path);
        &self.current_path
    }

    /*
    pub async fn execute(&mut self, app_name: &str, _args: Option<Vec<String>>, ) -> Result<String, String> {
        let app = self
            .apps
            .iter()
            .find(|app| app.name == app_name)
            .ok_or_else(|| format!("Commande introuvable: {}", app_name))?;

        if !app.executable {
            return Ok("Not executable".to_string());
        }

        let output = nwtzlang::interpreter_to_vec_string(&mut self.env, app.content.clone());
        Ok(format!("{:#?}", output))
    }
    */    
    
    
    fn find_directory<'a>(entries: &'a [FsEntry], path_parts: &[&str], ) -> Option<&'a Directory> {
        if path_parts.is_empty() {
            return None;
        }
        let current = path_parts[0];
        let rest = &path_parts[1..];
        for entry in entries {
            if let FsEntry::Directory(dir) = entry {
                if dir.name == current {
                    return if rest.is_empty() {
                        Some(dir)
                    } else {
                        Self::find_directory(&dir.content, rest)
                    };
                }
            }
        }
        None
    }

    pub fn rd(&self, path: &Path) -> Result<String, String> {
        let parts = split_components(path);
        if parts.is_empty() {
            return Err("Chemin vide".into());
        }

        let (dir_parts, file_name) = parts.split_at(parts.len() - 1);
        let container = if dir_parts.is_empty() {
            &self.root_content
        } else if let Some(dir) = Self::find_directory(&self.root_content, dir_parts) {
            &dir.content
        } else {
            return Err(format!("Répertoire introuvable : '{}'", path.display()));
        };

        for entry in container {
            if let FsEntry::File(f) = entry {
                if f.name == file_name[0] {
                    return Ok(f.content.clone());
                }
            }
        }
        Err(format!("Fichier introuvable : '{}'", path.display()))
    }

    #[warn(dead_code)]
    pub fn wr(&mut self, path: &Path, content: String) -> Result<(), String> {
        let parts = split_components(path);

        if parts.is_empty() {
            return Err("Chemin vide".into());
        }

        let (dir_parts, file_name) = parts.split_at(parts.len() - 1);

        let target_dir = if dir_parts.is_empty() {
            &mut self.root_content
        } else if let Some(dir) =
            Self::find_directory_mut(&mut self.root_content, dir_parts)
        {
            &mut dir.content
        } else {
            return Err(format!("Répertoire introuvable : '{}'", path.display()));
        };

        for entry in target_dir.iter_mut() {
            if let FsEntry::File(f) = entry {
                if f.name == file_name[0] {
                    f.content = content;
                    return Ok(());
                }
            }
        }

        let new_file = File {
            name: file_name[0].to_string(),
            content,
            //executable: false,
            //args: None,
            system: false,
        };
        target_dir.push(FsEntry::File(new_file));
        Ok(())
    }

    //pub fn find_file(&mut self, ,name: &str) -> String{
    //    
    //}


}

pub fn split_components(path: &Path) -> Vec<&str> {
    use std::path::Component;
    path.components()
        .filter_map(|c| {
            if let Component::Normal(os_str) = c {
                os_str.to_str()
            } else {
                None
            }
        })
        .collect()
}

pub fn init_system() -> Glob1Env {
    let mut env = Glob1Env {
        current_path: PathBuf::from("/"),
        root_content: Vec::new(),
        //command_input: String::new(),
        lang_env: Environment::new(None),
        inbox: Arc::new(UiInbox::new()),
        execution_started: false,
        message_buffer: Vec::new(),
    };

    env.add_entry_to_path(Path::new("/"), FsEntry::Directory(Directory {
        name: "sys".to_string(),
        content: Vec::new(),
        system: true,
    })).unwrap();
    
    env.add_entry_to_path(Path::new("/"), FsEntry::File(File {
        name: "Welcome.md".to_string(),
        content: r#"Welcome to glob1env !
made by Nyutiz"#.to_string(),
        system: false,
    })).unwrap();

    env.add_entry_to_path(Path::new("/"), FsEntry::File(File {
        name: "password.pwd".to_string(),
        content:
        r#"[PWD] Google::SuperPassword"#.to_string(),
        system: false,
    })).unwrap();

    env.add_entry_to_path(Path::new("/"), FsEntry::File(File {
        name: "password.nwtz!".to_string(),
        content:
        r#"ui.password("GLOBAL", "Put41n2m3r63-!...?44");
        ui.password("Google01", "Put41n2m3r63-!GOOG01?44");
        "#.to_string(),
        system: false,
    })).unwrap();

    env

}
