use egui::{Color32, Layout, TextStyle};
use egui_extras::{Size, StripBuilder};

use super::query_parser::parse_user_query;

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
enum SearchMode {
    Any,
    AtLeast,
    Only,
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
enum RadiatonType {
    Gamma,
    Alpha,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    //#[serde(skip)] // This how you opt-out of serialization of a field
    user_query: String,
    search_mode: SearchMode,
    message_to_user: String,
    search_results: String,
    radiation_type: RadiatonType,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            user_query: "Enter radiation energy or explore the given examples".to_string(),
            search_mode: SearchMode::Any,
            message_to_user: "Waiting for input".to_string(),
            search_results: "No results".to_string(),
            radiation_type: RadiatonType::Gamma,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let is_web = cfg!(target_arch = "wasm32");
            if is_web {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.heading("Decay Radiation Search");
                });
            }

            let dark_mode = ui.visuals().dark_mode;
            let faded_color = ui.visuals().window_fill();
            let faded_color = |color: Color32| -> Color32 {
                use egui::Rgba;
                let t = if dark_mode { 0.95 } else { 0.8 };
                egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
            };
            let body_text_size = TextStyle::Body.resolve(ui.style()).size;

            StripBuilder::new(ui)
                .size(Size::exact(3.0 * body_text_size)) // Examples bar
                .size(Size::relative(0.25)) // Query area
                .size(Size::exact(3.0 * body_text_size)) // Search options
                .size(Size::remainder()) // Results area
                .size(Size::exact(3.0 * body_text_size))
                .vertical(|mut strip| {
                    // Examples bar
                    strip.cell(|ui| {
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
                            0.0,
                            faded_color(Color32::BLUE),
                        );
                        ui.label("Examples?");
                    });
                    // Query area
                    strip.cell(|ui| {
                        ui.centered_and_justified(|ui| {
                            let user_query_response = ui.text_edit_multiline(&mut self.user_query);
                        });
                    });
                    // Search options
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label("Type: ");
                            ui.radio_value(&mut self.radiation_type, RadiatonType::Gamma, "Gamma");
                            ui.radio_value(&mut self.radiation_type, RadiatonType::Alpha, "Alpha");
                            ui.horizontal(|ui| ui.separator());
                            ui.label("Search mode: ");
                            ui.radio_value(&mut self.search_mode, SearchMode::Any, "Any");
                            ui.radio_value(&mut self.search_mode, SearchMode::AtLeast, "At least");
                            ui.radio_value(&mut self.search_mode, SearchMode::Only, "Only");
                            ui.horizontal(|ui| ui.separator());
                            let search_response = ui.button("Search");
                            if search_response.clicked() {
                                log::debug!("asdasaasdad");
                                self.search_results = parse_user_query(self.user_query.clone());
                            }
                        });
                    });
                    // Results area
                    strip.cell(|ui| {
                        ui.centered_and_justified(|ui| {
                            let result_response = ui.text_edit_multiline(&mut self.search_results);
                        });
                    });
                    strip.cell(|ui| {
                        ui.separator();

                        ui.hyperlink_to(
                            "Source code",
                            "https://github.com/cristian-jfv/decay-radiation-search",
                        );

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            egui::warn_if_debug_build(ui);
                        });
                    });
                });
        });
    }
}
