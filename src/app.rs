use egui::{Color32, TextStyle};
use egui_extras::{Size, StripBuilder};

use crate::query_parser::search_energies;

const GAMMA_EXAMPLE_STRING: &str = "# This is a comment and is not considered for the query
# The energy can be in eV, keV, and MeV

6.96 keV 1% # Uncertainty is expressed in percentage
215.9 keV 1%
231.6 keV 0.5%
0.2389 MeV 0.5%

# It is possible to show all radiation records from a decay dataset or only the ones that match the query";

const ALPHA_EXAMPLE_STRING: &str = "4.149 MeV 0.5%
4.198 MeV 1%";

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum PrintMode {
    Everything,
    OnlyMatches,
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum RadiationType {
    Gamma,
    Alpha,
}

impl PartialEq<String> for RadiationType {
    fn eq(&self, other: &String) -> bool {
        matches!(
            (self, other.as_str()),
            (RadiationType::Gamma, "G") | (RadiationType::Alpha, "A")
        )
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    //#[serde(skip)] // This how you opt-out of serialization of a field
    user_query: String,
    print_mode: PrintMode,
    message_to_user: String,
    search_results: String,
    radiation_type: RadiationType,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            user_query: "Enter radiation energy or explore the given examples".to_string(),
            print_mode: PrintMode::OnlyMatches,
            message_to_user: "Waiting for input".to_string(),
            search_results: "No results".to_string(),
            radiation_type: RadiationType::Gamma,
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
            let _faded_color = |color: Color32| -> Color32 {
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
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            if ui.button("Gamma example").clicked() {
                                self.user_query = GAMMA_EXAMPLE_STRING.to_string();
                                self.radiation_type = RadiationType::Gamma;
                                self.search_results = search_energies(
                                    self.user_query.clone(),
                                    &self.radiation_type,
                                    &self.print_mode,
                                );
                            }
                            if ui.button("Alpha example").clicked() {
                                self.user_query = ALPHA_EXAMPLE_STRING.to_string();
                                self.radiation_type = RadiationType::Alpha;
                                self.search_results = search_energies(
                                    self.user_query.clone(),
                                    &self.radiation_type,
                                    &self.print_mode,
                                );
                            }
                        });
                    });
                    // Query area
                    strip.cell(|ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                let _user_query_response =
                                    ui.text_edit_multiline(&mut self.user_query);
                            })
                        });
                    });
                    // Search options
                    strip.cell(|ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.label("Type: ");
                                ui.radio_value(
                                    &mut self.radiation_type,
                                    RadiationType::Gamma,
                                    "Gamma",
                                );
                                ui.radio_value(
                                    &mut self.radiation_type,
                                    RadiationType::Alpha,
                                    "Alpha",
                                );
                                ui.horizontal(|ui| ui.separator());
                                ui.label("Show: ");
                                ui.radio_value(
                                    &mut self.print_mode,
                                    PrintMode::OnlyMatches,
                                    "only matches",
                                );
                                ui.radio_value(
                                    &mut self.print_mode,
                                    PrintMode::Everything,
                                    "everything",
                                );
                                ui.horizontal(|ui| ui.separator());
                                let search_response = ui.button("Search");
                                if search_response.clicked() {
                                    self.search_results = search_energies(
                                        self.user_query.clone(),
                                        &self.radiation_type,
                                        &self.print_mode,
                                    );
                                }
                            })
                        });
                    });
                    // Results area
                    strip.cell(|ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                //let result_response =
                                //   ui.text_edit_multiline(&mut self.search_results);
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.search_results)
                                        .font(egui::TextStyle::Monospace),
                                )
                            })
                        });
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Data source ENSDF 240402:");
                            ui.hyperlink("https://www.nndc.bnl.gov/ensdfarchivals/");
                            ui.label("Source code:");
                            ui.hyperlink("https://github.com/cristian-jfv/decay-radiation-search");
                        });

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            egui::warn_if_debug_build(ui);
                        });
                    });
                });
        });
    }
}
