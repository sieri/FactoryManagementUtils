use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use copypasta::{ClipboardContext, ClipboardProvider};
use egui::{Context, Widget};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

use crate::app::recipe_window::arrow_flow::ArrowFlow;
use crate::app::recipe_window::basic_recipe_window_descriptor::BasicRecipeWindowDescriptor;
use crate::app::recipe_window::resource_sink::ResourceSink;
use crate::app::recipe_window::resources_sources::ResourceSource;
use crate::app::resources::resource_flow::ResourceFlow;
use crate::app::resources::ManageFlow;
use crate::utils::Io;
use commons::CommonsManager;
use eframe::Frame;
use error::ShowError;
use resources::recipe_input_resource::RecipeInputResource;
use resources::recipe_output_resource::RecipeOutputResource;
use resources::resource_flow::ManageResourceFlow;
use std::time::Duration;

pub mod commons;
pub mod coordinates_info;
pub mod error;
pub mod recipe_window;
pub(crate) mod resources;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FactoryManagementUtilsApp {
    new_recipe_title: String,
    new_resource_source: String,
    recipes: Vec<BasicRecipeWindowDescriptor>,
    sources: Vec<ResourceSource>,
    sinks: Vec<ResourceSink>,
    #[serde(skip)]
    commons: CommonsManager,

    active_arrow: Option<ArrowFlow>,
    arrows: Vec<ArrowFlow>,
}

#[derive(Copy, Clone)]
enum FlowCalculatorType {
    Helper(FlowCalculatorHelper),
    EndRecipe(usize),
}

#[derive(Copy, Clone)]
struct FlowCalculatorHelper {
    source_window_index: usize,
    source_flow_index: usize,

    end_window_index: usize,
    end_flow_index: usize,

    source_type: RecipeWindowType,
    end_type: RecipeWindowType,
}

impl Default for FactoryManagementUtilsApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            new_recipe_title: "Hello World!".to_owned(),
            new_resource_source: "".to_string(),
            recipes: vec![],
            sources: vec![],
            sinks: vec![],
            commons: CommonsManager {
                window_coordinates: Default::default(),
                arrow_active: false,
                clicked_start_arrow_info: None,
                clicked_place_arrow_info: None,
                recalculate: false,
                show_errors: Default::default(),
                show_tooltips: Default::default(),
            },
            active_arrow: None,
            arrows: vec![],
        }
    }
}

impl FactoryManagementUtilsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut loaded: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            loaded.calculate();
            return loaded;
        }

        Default::default()
    }

    ///reload
    fn reload(&mut self, other: Self) {
        self.new_recipe_title = other.new_recipe_title;
        self.new_resource_source = other.new_resource_source;
        self.recipes = other.recipes;
        self.sources = other.sources;
        self.sinks = other.sinks;
        self.active_arrow = other.active_arrow;
        self.arrows = other.arrows;
    }

    fn calculate(&mut self) {
        println!("==================Calculate==================");

        let mut sources_helpers = LinkedList::new();
        let mut sinks_helpers = LinkedList::new();
        let mut recipes_helpers = vec![(0, LinkedList::new()); self.recipes.len()];
        //reset flows
        for source in self.sources.iter_mut() {
            source.output.reset();
        }

        for recipe in self.recipes.iter_mut() {
            for input in recipe.inputs.iter_mut() {
                match input {
                    ManageFlow::RecipeInput(r) => r.reset(),
                    ManageFlow::RecipeOutput(_) => {}
                }
            }
            for output in recipe.outputs.iter_mut() {
                match output {
                    ManageFlow::RecipeInput(_) => {}
                    ManageFlow::RecipeOutput(r) => r.reset(),
                }
            }
        }

        for sink in self.sinks.iter_mut() {
            if let Some(f) = sink.sink.as_mut() {
                f.reset();
            }
        }

        //build relationships from arrows
        for arrow in self.arrows.iter() {
            let start_id = arrow.start_flow_window;
            let source_flow_index = arrow.start_flow_index;
            let source_type = arrow.start_flow_type;

            let source_window_index = match source_type {
                RecipeWindowType::Basic => {
                    self.recipes.iter().position(|recipe| recipe.id == start_id)
                }
                RecipeWindowType::Source => {
                    self.sources.iter().position(|recipe| recipe.id == start_id)
                }
                RecipeWindowType::Sink => {
                    println!("incorrect source type");
                    None
                }
            };

            let end_id = arrow
                .end_flow_window
                .unwrap_or_else(|| egui::Id::new("Invalid ID"));
            let end_flow_index = arrow.end_flow_index;
            let end_type = arrow.end_flow_type.unwrap_or(RecipeWindowType::Source);
            let end_window_index = match end_type {
                RecipeWindowType::Basic => {
                    self.recipes.iter().position(|recipe| recipe.id == end_id)
                }
                RecipeWindowType::Source => None,
                RecipeWindowType::Sink => self.sinks.iter().position(|sink| sink.id == end_id),
            };

            if let Some(source_window_index) = source_window_index {
                if let Some(end_window_index) = end_window_index {
                    let helper = FlowCalculatorHelper {
                        source_window_index,
                        source_flow_index,
                        end_window_index,
                        end_flow_index,

                        source_type,
                        end_type,
                    };

                    match source_type {
                        RecipeWindowType::Basic => {
                            let source_order = recipes_helpers[source_window_index].0;
                            let mut end_order = recipes_helpers[end_window_index].0;

                            if source_order >= end_order {
                                end_order = source_order + 1usize;
                            }

                            recipes_helpers[end_window_index]
                                .1
                                .push_back(FlowCalculatorType::Helper(helper));
                            recipes_helpers[end_window_index].0 = end_order;
                        }
                        RecipeWindowType::Source => {
                            sources_helpers.push_back(FlowCalculatorType::Helper(helper));
                        }
                        RecipeWindowType::Sink => {
                            sinks_helpers.push_back(FlowCalculatorType::Helper(helper))
                        }
                    }
                }
            }
        }
        let mut calculate_helper = sources_helpers;

        recipes_helpers.sort_by(|helper1, helper2| helper1.0.partial_cmp(&helper2.0).unwrap());

        for (order, mut list) in recipes_helpers {
            println!("{order}");
            if list.front().is_some() {
                let index = match list.front().unwrap() {
                    FlowCalculatorType::Helper(h) => h.end_window_index,
                    FlowCalculatorType::EndRecipe(i) => *i,
                };
                list.push_back(FlowCalculatorType::EndRecipe(index));
                calculate_helper.append(&mut list);
            }
        }

        calculate_helper.append(&mut sinks_helpers);

        //calculate
        for calculate_helper in calculate_helper.iter_mut() {
            match calculate_helper {
                FlowCalculatorType::Helper(h) => match h.source_type {
                    RecipeWindowType::Basic => {
                        let source = &mut self.recipes[h.source_window_index];
                        let source_flow = &mut source.outputs[h.source_flow_index];
                        match source_flow {
                            ManageFlow::RecipeInput(_) => {
                                println!("Resource source wrong")
                            }
                            ManageFlow::RecipeOutput(o) => {
                                let used_flow = o.created.clone();
                                let added_source = o.add_out_flow(used_flow.clone());

                                let end_flow = match h.end_type {
                                    RecipeWindowType::Basic => {
                                        let end = &mut self.recipes[h.end_window_index];
                                        let f = match &mut end.inputs[h.end_flow_index] {
                                            ManageFlow::RecipeInput(r) => r,
                                            ManageFlow::RecipeOutput(_) => {
                                                panic!("Never happens")
                                            }
                                        };
                                        Some(f)
                                    }
                                    RecipeWindowType::Source => None,
                                    RecipeWindowType::Sink => {
                                        let end = &mut self.sinks[h.end_window_index];
                                        let f = end.sink.as_mut();
                                        match f {
                                            None => {
                                                let f = RecipeInputResource::new(
                                                    used_flow.resource.clone(),
                                                    used_flow.clone(),
                                                );
                                                end.sink = Some(f);
                                                let f = end.sink.as_mut().unwrap();
                                                Some(f)
                                            }
                                            Some(f) => Some(f),
                                        }
                                    }
                                }
                                .unwrap();

                                let added_input = end_flow.add_in_flow(used_flow);

                                if !(added_source && added_input) {
                                    println!("Error: added_source:{added_source} added_inputs{added_input}");
                                }
                            }
                        }
                    }
                    RecipeWindowType::Source => {
                        let source = &mut self.sources[h.source_window_index];
                        let end_flow = match h.end_type {
                            RecipeWindowType::Basic => {
                                let end = &mut self.recipes[h.end_window_index];
                                let f = match &mut end.inputs[h.end_flow_index] {
                                    ManageFlow::RecipeInput(r) => r,
                                    ManageFlow::RecipeOutput(_) => {
                                        panic!("Never happens")
                                    }
                                };
                                Some(f)
                            }
                            RecipeWindowType::Source => None,
                            RecipeWindowType::Sink => {
                                let end = &mut self.sinks[h.end_window_index];
                                let f = end.sink.as_mut().unwrap();
                                Some(f)
                            }
                        };
                        if let Some(end_flow) = end_flow {
                            if source.limited_output {
                                let used_flow = source.output.created.clone();
                                add_flows(&mut source.output, end_flow, used_flow);
                            } else {
                                let used_flow = end_flow.needed.clone();
                                add_flows(&mut source.output, end_flow, used_flow);
                            }
                        }
                    }
                    RecipeWindowType::Sink => {}
                },

                FlowCalculatorType::EndRecipe(_) => {}
            }
        }
    }

    fn top_panel(&mut self, ctx: &Context, _frame: &mut Frame, error: bool) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Save").clicked() {
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
                                return;
                            }
                        };

                        let r = self.serialize(&mut serde_json::Serializer::new(file_write));

                        if let Err(e) = r {
                            self.commons.add_error(ShowError {
                                error: e.to_string(),
                                context: "The save failed for the following reason".to_string(),
                            })
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Load").clicked() {
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
                                return;
                            }
                        };

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
                    if ui.button("Reset").clicked() {
                        self.new_recipe_title.clear();
                        self.recipes.clear();
                        self.arrows.clear();
                        self.sources.clear();
                        self.sinks.clear();
                    }
                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });
    }

    fn side_panel(&mut self, ctx: &Context, error: bool) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            ui.heading("Control panel");

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
                    let recipe_window =
                        BasicRecipeWindowDescriptor::new(self.new_recipe_title.to_owned());
                    self.recipes.push(recipe_window);
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("New resource source:");
                ui.text_edit_singleline(&mut self.new_resource_source);
            });

            if ui.button("Create source").clicked() {
                let source = ResourceSource::new(self.new_resource_source.clone());
                self.sources.push(source);
            }
            if ui.button("Create sink").clicked() {
                let sink = ResourceSink::new();
                self.sinks.push(sink);
            }

            ui.separator();
            ui.collapsing("Resource usage", |ui| {
                for source in self.sources.iter() {
                    let flow = source.output.total_out();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}: {} {}",
                            flow.resource.name, flow.amount, flow.rate
                        ));
                    });
                }
            });
            ui.collapsing("Resource generated", |ui| {
                for sink in self.sinks.iter() {
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

            self.debug(ui);

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
        });
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

            self.recipes
                .retain_mut(|recipe| recipe.show(&mut self.commons, ctx, !error));
            self.sources
                .retain_mut(|source| source.show(&mut self.commons, ctx, !error));
            self.sinks
                .retain_mut(|sink| sink.show(&mut self.commons, ctx, !error));

            self.arrow_management(ctx, error);
        });
    }

    fn arrow_management(&mut self, ctx: &Context, error: bool) {
        self.arrows
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

                            self.arrows
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
}

fn add_flows(
    source: &mut RecipeOutputResource<usize>,
    input: &mut RecipeInputResource<usize>,
    used_flow: ResourceFlow<usize, f32>,
) {
    let added_source = source.add_out_flow(used_flow.clone());
    let added_input = input.add_in_flow(used_flow);

    if !(added_source && added_input) {
        println!("Error: added_source:{added_source} added_inputs{added_input}");
    }
}

impl eframe::App for FactoryManagementUtilsApp {
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
            self.calculate();
            self.commons.recalculate = false;
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl FactoryManagementUtilsApp {
    ///Show a debug area
    fn debug(&mut self, ui: &mut egui::Ui) {
        if cfg!(debug_assertions) {
            ui.separator();

            ui.collapsing("DEBUG", |ui| {
                let mut a = self.commons.arrow_active;
                ui.checkbox(&mut a, "Arrow active");
                let mut start_some = self.commons.clicked_start_arrow_info.is_some();
                let mut place_some = self.commons.clicked_place_arrow_info.is_some();
                ui.checkbox(&mut start_some, "clicked start is some");
                ui.checkbox(&mut place_some, "clicked place is some");
                let mut arrow_count = self.arrows.len();
                ui.horizontal(|ui| {
                    ui.label("Arrow");
                    egui::DragValue::new(&mut arrow_count).ui(ui);
                });
                let mut recipe_count = self.recipes.len();
                ui.horizontal(|ui| {
                    ui.label("Recipe");
                    egui::DragValue::new(&mut recipe_count).ui(ui);
                });
                let mut sources_count = self.sources.len();
                ui.horizontal(|ui| {
                    ui.label("Source");
                    egui::DragValue::new(&mut sources_count).ui(ui);
                });
                let mut sinks_count = self.sinks.len();
                ui.horizontal(|ui| {
                    ui.label("Sink");
                    egui::DragValue::new(&mut sinks_count).ui(ui);
                });

                if ui.button("Calculate").clicked() {
                    self.calculate();
                }

                if ui.button("Update all").clicked() {
                    self.calculate();
                    for recipe in self.recipes.iter_mut() {
                        recipe.update_flow(Io::Input).expect("Update failure input");
                        recipe
                            .update_flow(Io::Output)
                            .expect("Update failure output");
                    }
                }
            });
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
