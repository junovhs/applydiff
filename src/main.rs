use eframe::egui;
use std::path::PathBuf;
use std::fs;

#[derive(Default)]
struct PachApp {
    selected_path: Option<PathBuf>,
    last_result: String,
}

impl eframe::App for PachApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pach - Diff Applier");
            
            ui.separator();
            
            // Directory selection
            if ui.button("📁 Select Directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.selected_path = Some(path);
                }
            }
            
            if let Some(path) = &self.selected_path {
                ui.label(format!("Selected: {}", path.display()));
            } else {
                ui.label("No directory selected");
            }
            
            ui.separator();
            
            // Apply patch button
            let can_apply = self.selected_path.is_some();
            
            if ui.add_enabled(can_apply, egui::Button::new("📋 Apply Patch from Clipboard")).clicked() {
                self.apply_patch_from_clipboard();
            }
            
            if !can_apply {
                ui.label("Select a directory first");
            }
            
            ui.separator();
            
            // Results
            if !self.last_result.is_empty() {
                ui.label("Last Result:");
                ui.text_edit_multiline(&mut self.last_result);
            }
        });
    }
}

impl PachApp {
    fn apply_patch_from_clipboard(&mut self) {
        let clipboard_text = match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
            Ok(text) => text,
            Err(e) => {
                self.last_result = format!("Failed to read clipboard: {}", e);
                return;
            }
        };
        
        if let Some(base_path) = &self.selected_path {
            match apply_patch(&clipboard_text, base_path) {
                Ok(files) => {
                    self.last_result = format!("✅ Successfully patched {} files:\n{}", files.len(), files.join("\n"));
                }
                Err(e) => {
                    self.last_result = format!("❌ Error: {}", e);
                }
            }
        }
    }
}

fn apply_patch(patch_content: &str, base_path: &PathBuf) -> Result<Vec<String>, String> {
    let mut files_patched = Vec::new();
    let lines: Vec<&str> = patch_content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        if lines[i].starts_with("--- ") && i + 1 < lines.len() && lines[i + 1].starts_with("+++ ") {
            let mut file_path = lines[i + 1][4..].trim().to_string();
            if file_path.starts_with("b/") {
                file_path = file_path[2..].to_string();
            }
            
            let full_path = base_path.join(&file_path);
            if !full_path.exists() {
                return Err(format!("File {} does not exist", file_path));
            }
            
            let original_content = fs::read_to_string(&full_path)
                .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
            
            let mut original_lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();
            let mut offset: i32 = 0;
            
            i += 2;
            
            while i < lines.len() && lines[i].starts_with("@@") {
                let hunk_header = lines[i];
                let parts: Vec<&str> = hunk_header.split_whitespace().collect();
                if parts.len() < 3 {
                    i += 1;
                    continue;
                }
                
                let old_range = parts[1];
                let old_start = if let Some(comma_pos) = old_range.find(',') {
                    old_range[1..comma_pos].parse::<i32>().unwrap_or(1) - 1
                } else {
                    old_range[1..].parse::<i32>().unwrap_or(1) - 1
                };
                
                i += 1;
                let mut old_line_num = (old_start + offset) as usize;
                
                while i < lines.len() && !lines[i].starts_with("@@") && !lines[i].starts_with("--- ") {
                    if lines[i].starts_with("-") {
                        if old_line_num < original_lines.len() {
                            original_lines.remove(old_line_num);
                            offset -= 1;
                        }
                    } else if lines[i].starts_with("+") {
                        let new_line = lines[i][1..].to_string();
                        if old_line_num <= original_lines.len() {
                            original_lines.insert(old_line_num, new_line);
                            old_line_num += 1;
                            offset += 1;
                        }
                    } else if lines[i].starts_with(" ") {
                        old_line_num += 1;
                    }
                    i += 1;
                }
            }
            
            let patched_content = original_lines.join("\n");
            fs::write(&full_path, patched_content)
                .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;
            
            files_patched.push(file_path);
        } else {
            i += 1;
        }
    }

    Ok(files_patched)
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Pach",
        options,
        Box::new(|_cc| Ok(Box::new(PachApp::default()))),
    )
}
