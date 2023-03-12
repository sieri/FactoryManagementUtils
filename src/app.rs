use crate::app::recipe_window::RecipeWindowGUI;
use copypasta::{ClipboardContext, ClipboardProvider};
use egui::{Context, Ui, Widget};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

use crate::app::recipe_graph::RecipeGraph;
use crate::app::recipe_window::arrow_flow::ArrowFlow;
use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;
use crate::app::recipe_window::resource_sink::ResourceSink;
use crate::app::recipe_window::resources_sources::ResourceSource;
use crate::app::recipe_window::simple_recipe_window::SimpleRecipeWindow;
use crate::utils::Io;
use commons::CommonsManager;
use eframe::Frame;
use error::ShowError;
use log::info;
use resources::resource_flow::ManageResourceFlow;
use std::time::Duration;

pub mod commons;
pub mod coordinates_info;
pub mod error;
mod recipe_graph;
pub mod recipe_window;
pub(crate) mod resources;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FactoryManagementApp {
    new_recipe_title: String,
    new_resource_source: String,
    current_graph: RecipeGraph,
    commons: CommonsManager,
    active_arrow: Option<ArrowFlow>,
}

impl Default for FactoryManagementApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            new_recipe_title: "Hello World!".to_owned(),
            new_resource_source: "".to_string(),
            commons: Default::default(),
            active_arrow: None,
            current_graph: RecipeGraph::new(),
        }
    }
}

impl FactoryManagementApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut loaded: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            info!("Initial calculations after load from app storage");
            loaded.current_graph.calculate();
            return loaded;
        }

        Default::default()
    }

    ///reload
    fn reload(&mut self, other: Self) {
        self.new_recipe_title = other.new_recipe_title;
        self.new_resource_source = other.new_resource_source;
        self.current_graph = other.current_graph;
        self.active_arrow = other.active_arrow;
    }

    fn top_panel(&mut self, ctx: &Context, _frame: &mut Frame, error: bool) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Save").clicked() {
                        let file_write = self.select_file_out();
                        if let Some(file_write) = file_write {
                            self.save(&file_write);
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Load").clicked() {
                        let file_read = self.file_select_in();
                        if let Some(file_read) = file_read {
                            self.load(file_read);
                        }
                    }
                    if ui.button("Reset").clicked() {
                        self.reset();
                    }
                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });
    }

    fn reset(&mut self) {
        self.new_recipe_title.clear();
        self.current_graph.clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn select_file_out(&mut self) -> Option<File> {
        let file = FileDialog::new()
            .add_filter("FactoryManagementUtils file", &["fmu"])
            .set_directory("/")
            .save_file()
            .unwrap();

        let file_write = File::create(file);

        let file_write = match file_write {
            Ok(f) => f,
            Err(e) => {
                self.commons.add_error(ShowError::new(e.to_string()));
                return None;
            }
        };
        Some(file_write)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn file_select_in(&mut self) -> Option<File> {
        let file = FileDialog::new()
            .add_filter("FactoryManagementUtils file", &["fmu"])
            .set_directory("/")
            .pick_file()
            .unwrap();

        let file_read = File::open(file);

        let file_read = match file_read {
            Ok(f) => f,
            Err(e) => {
                self.commons.add_error(ShowError::new(e.to_string()));
                return None;
            }
        };
        Some(file_read)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load(&mut self, file_read: File) {
        let mut deserializer = serde_json::Deserializer::from_reader(file_read);
        let r = Self::deserialize(&mut deserializer);

        match r {
            Ok(factory_app) => {
                self.reload(factory_app);
            }
            Err(e) => self.commons.add_error(ShowError {
                error: e.to_string(),
                context: "The save failed for the following reason".to_string(),
            }),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self, file_write: &File) {
        let r = self.serialize(&mut serde_json::Serializer::new(file_write));

        if let Err(e) = r {
            self.commons.add_error(ShowError {
                error: e.to_string(),
                context: "The save failed for the following reason".to_string(),
            })
        }
    }

    fn side_panel(&mut self, ctx: &Context, error: bool) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            ui.heading("Control panel");

            self.recipe_adding(ui);

            ui.separator();

            self.sources_and_sinks_adding(ui);

            ui.separator();
            self.resource_usage(ui);
            self.resource_generation(ui);

            ui.separator();
            self.recipes_list(ui);

            ui.separator();
            self.compounds_recipes(ui);

            self.debug(ui);

            Self::left_panel_footer(ui);
        });
    }

    fn left_panel_footer(ui: &mut Ui) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        });
    }

    fn recipes_list(&mut self, ui: &mut Ui) {
        ui.label("Recipe lists:");
        self.commons.saved_recipes.recipes_list(ui);
        if ui.button("Spawn recipes").clicked() {
            let r = self.commons.saved_recipes.load();
            match r {
                Ok(data) => self.current_graph.simple_recipes.push(data),
                Err(e) => self.commons.add_error(e),
            };
        }
        ui.horizontal(|ui| {
            if ui.button("Save all recipes").clicked() {
                for recipe in self.current_graph.simple_recipes.iter_mut() {
                    self.commons.save(recipe);
                }
            }
            if ui.button("Clear all recipes").clicked() {
                self.commons.saved_recipes.clear();
            }
        });
    }

    fn resource_generation(&mut self, ui: &mut Ui) {
        ui.collapsing("Resource generated", |ui| {
            for sink in self.current_graph.sinks.iter() {
                if let Some(sink) = sink.sink.as_ref() {
                    let flow = sink.total_in();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}: {} {}",
                            flow.resource.name, flow.amount, flow.rate
                        ));
                    });
                }
            }
        });
    }

    fn resource_usage(&mut self, ui: &mut Ui) {
        ui.collapsing("Resource usage", |ui| {
            for source in self.current_graph.sources.iter() {
                let flow = source.output.total_out();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{}: {} {}",
                        flow.resource.name, flow.amount, flow.rate
                    ));
                });
            }
        });
    }

    fn sources_and_sinks_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("New resource source:");
            ui.text_edit_singleline(&mut self.new_resource_source);
        });

        if ui.button("Create source").clicked() {
            let source = ResourceSource::new(self.new_resource_source.clone());
            self.current_graph.sources.push(source);
        }
        if ui.button("Create sink").clicked() {
            let sink = ResourceSink::new();
            self.current_graph.sinks.push(sink);
        }
    }

    fn recipe_adding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("New recipe title: ");
            ui.text_edit_singleline(&mut self.new_recipe_title);
        });

        if ui.button("Create recipe").clicked() {
            //spawn a button in the central panel
            if self.new_recipe_title.is_empty() {
                self.commons
                    .add_error(ShowError::new("Need a title to create a window".to_owned()))
            } else {
                let recipe_window = SimpleRecipeWindow::new(self.new_recipe_title.to_owned());
                self.current_graph.simple_recipes.push(recipe_window);
            }
        }
    }

    fn central_panel(&mut self, ctx: &Context, error: bool) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.set_enabled(!error);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.heading("Factory Management Utils");
                ui.hyperlink("https://github.com/sieri/FactoryManagementUtils");
                egui::warn_if_debug_build(ui);
            });

            self.current_graph
                .simple_recipes
                .retain_mut(|recipe| recipe.show(&mut self.commons, ctx, !error));
            self.current_graph
                .compound_recipes
                .retain_mut(|compound_recipe| compound_recipe.show(&mut self.commons, ctx, !error));
            self.current_graph
                .sources
                .retain_mut(|source| source.show(&mut self.commons, ctx, !error));
            self.current_graph
                .sinks
                .retain_mut(|sink| sink.show(&mut self.commons, ctx, !error));

            self.arrow_management(ui, ctx, error);
        });
    }

    fn arrow_management(&mut self, _ui: &mut Ui, ctx: &Context, error: bool) {
        self.current_graph
            .arrows
            .retain_mut(|arrow| arrow.show(&mut self.commons, ctx, !error));
        if self.active_arrow.is_some() {
            let active = self
                .active_arrow
                .as_mut()
                .unwrap()
                .show(&mut self.commons, ctx, !error);

            if active {
                if self.commons.clicked_place_arrow_info.is_some() {
                    let (resource, id, flow_index, recipe_type) =
                        self.commons.clicked_place_arrow_info.as_ref().unwrap();
                    let err = self.active_arrow.as_mut().unwrap().put_end(
                        resource.clone(),
                        *id,
                        *recipe_type,
                        *flow_index,
                    );

                    match err {
                        Ok(_) => {
                            //connect arrow
                            self.current_graph
                                .arrows
                                .push(self.active_arrow.as_ref().unwrap().clone());
                        }
                        Err(e) => self.commons.add_error(ShowError::new(e.str())),
                    };

                    self.active_arrow = None;
                    self.commons.arrow_active = false;
                    self.commons.recalculate = true;
                    self.commons.clicked_place_arrow_info = None;
                }
            } else {
                self.active_arrow = None;
                self.commons.arrow_active = false;
            }
        } else if self.commons.clicked_start_arrow_info.is_some() {
            let (resource, id, layer, flow_index, recipe_type) =
                self.commons.clicked_start_arrow_info.as_ref().unwrap();
            self.active_arrow = Some(ArrowFlow::new(
                resource.clone(),
                *id,
                *recipe_type,
                *layer,
                *flow_index,
            ));
            self.commons.clicked_start_arrow_info = None;
        }
    }
    fn compounds_recipes(&mut self, ui: &mut Ui) {
        if ui.button("Create Compound Recipe").clicked() {
            let compound_graph = CompoundRecipeWindow::new(self.current_graph.clone());
            self.current_graph = RecipeGraph::new();
            self.current_graph.compound_recipes.push(compound_graph);
        }
    }
}

impl eframe::App for FactoryManagementApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let error = !self.commons.show_errors.is_empty();

        if error {
            self.error_window(ctx);
        }

        self.top_panel(ctx, _frame, error);

        self.side_panel(ctx, error);

        self.central_panel(ctx, error);

        if self.commons.recalculate {
            self.update_flows();
            self.current_graph.calculate();
            self.commons.recalculate = false;
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl FactoryManagementApp {
    ///Show a debug area
    fn debug(&mut self, ui: &mut Ui) {
        if cfg!(debug_assertions) {
            ui.separator();

            ui.collapsing("DEBUG", |ui| {
                let mut a = self.commons.arrow_active;
                ui.checkbox(&mut a, "Arrow active");
                let mut start_some = self.commons.clicked_start_arrow_info.is_some();
                let mut place_some = self.commons.clicked_place_arrow_info.is_some();
                ui.checkbox(&mut start_some, "clicked start is some");
                ui.checkbox(&mut place_some, "clicked place is some");
                let mut arrow_count = self.current_graph.arrows.len();
                ui.horizontal(|ui| {
                    ui.label("Arrow");
                    egui::DragValue::new(&mut arrow_count).ui(ui);
                });
                let mut recipe_count = self.current_graph.simple_recipes.len();
                ui.horizontal(|ui| {
                    ui.label("Recipe");
                    egui::DragValue::new(&mut recipe_count).ui(ui);
                });
                let mut sources_count = self.current_graph.sources.len();
                ui.horizontal(|ui| {
                    ui.label("Source");
                    egui::DragValue::new(&mut sources_count).ui(ui);
                });
                let mut sinks_count = self.current_graph.sinks.len();
                ui.horizontal(|ui| {
                    ui.label("Sink");
                    egui::DragValue::new(&mut sinks_count).ui(ui);
                });

                if ui.button("Calculate").clicked() {
                    info!("Calculate button pressed");
                    self.current_graph.calculate();
                }
            });
        }
    }

    fn update_flows(&mut self) {
        for recipe in self.current_graph.simple_recipes.iter_mut() {
            recipe
                .inner_recipe
                .update_flow(Io::Input)
                .expect("Update failure input");
            recipe
                .inner_recipe
                .update_flow(Io::Output)
                .expect("Update failure output");
        }
        for recipe in self.current_graph.simple_recipes.iter_mut() {
            recipe
                .inner_recipe
                .update_flow(Io::Input)
                .expect("Update failure input");
            recipe
                .inner_recipe
                .update_flow(Io::Output)
                .expect("Update failure output");
        }
    }

    /// Show error window.
    fn error_window(&mut self, ctx: &Context) -> bool {
        let err = self.commons.show_errors.pop_back();
        if let Some(err) = err {
            let mut result = true;
            let width = 550.0;
            let height = 185.0;
            let red = egui::Color32::from_rgb(210, 40, 40);

            egui::Window::new("Error")
                .collapsible(false)
                .default_pos((100.0, 100.0))
                .fixed_size((width, height))
                .show(ctx, |ui| {
                    ui.label(&err.context);

                    egui::ScrollArea::new([true, false])
                        .max_height(height)
                        .show(ui, |ui| {
                            egui::TextEdit::multiline(&mut err.error.to_string())
                                .interactive(false)
                                .font(egui::TextStyle::Monospace)
                                .text_color(red)
                                .desired_width(width)
                                .desired_rows(10)
                                .show(ui);
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        let tooltip_id = egui::Id::new("error-copypasta");

                        if ui.button("Copy to Clipboard").clicked() {
                            let mut copied = false;
                            if let Ok(mut clipboard) = ClipboardContext::new() {
                                copied = clipboard.set_contents(err.error.to_string()).is_ok();
                            }

                            let label = if copied {
                                "Copied!"
                            } else {
                                "Sorry, but the clipboard isn't working..."
                            };

                            self.commons.add_tooltip(tooltip_id, label.to_string());
                        }

                        // Show the copy button tooltip for 3 seconds
                        self.commons
                            .tooltip(ctx, ui, tooltip_id, Duration::from_secs(3));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("Okay").clicked() {
                                result = false;
                            } else {
                                self.commons.show_errors.push_back(err);
                            }
                        });
                    });
                });

            result
        } else {
            true
        }
    }
}

#[cfg(test)]
pub mod tests {}
