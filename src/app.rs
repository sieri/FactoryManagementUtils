use crate::recipe_window::{BasicRecipeWindowDescriptor, RecipeWindowGUI};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FactoryManagementUtilsApp {
    // Example stuff:
    label: String,
    recipes: Vec<BasicRecipeWindowDescriptor>,

    // List of error popups to keep
    #[serde(skip)]
    show_errors: VecDeque<ShowError>,

    /// List of tooltips that can be shown
    #[serde(skip)]
    show_tooltips: HashMap<egui::Id, (String, Instant)>,
}

impl Default for FactoryManagementUtilsApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            recipes: vec![],
            show_errors: Default::default(),
            show_tooltips: Default::default(),
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
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for FactoryManagementUtilsApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let error = !self.show_errors.is_empty();

        if error {
            self.error_window(ctx);
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Reset").clicked() {
                        self.label.clear();
                        self.recipes.clear();
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.set_enabled(!error);
            ui.heading("Control panel");

            ui.horizontal(|ui| {
                ui.label("New recipe title: ");
                ui.text_edit_singleline(&mut self.label);
            });

            if ui.button("Create recipe").clicked() {
                //spawn a button in the central panel
                if self.label.is_empty() {
                    self.add_error(ShowError::new("Need a title to create a window".to_owned()))
                } else {
                    let recipe_window = BasicRecipeWindowDescriptor::new(self.label.to_owned());
                    self.recipes.push(recipe_window);
                }
            }

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

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.set_enabled(!error);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.heading("Factory Management Utils");
                ui.hyperlink("https://github.com/sieri/FactoryManagementUtils");
                egui::warn_if_debug_build(ui);
            });

            self.recipes.retain(|recipe| recipe.show(ctx, !error));
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl FactoryManagementUtilsApp {
    /// Show error window.
    fn error_window(&mut self, ctx: &egui::Context) -> bool {
        let err = self.show_errors.pop_back();
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

                            self.add_tooltip(tooltip_id, label);
                        }

                        // Show the copy button tooltip for 3 seconds
                        self.tooltip(ctx, ui, tooltip_id, Duration::from_secs(3));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("Okay").clicked() {
                                result = false;
                            } else {
                                self.show_errors.push_back(err);
                            }
                        });
                    });
                });

            result
        } else {
            true
        }
    }

    /// Add an error to the GUI.
    ///
    /// The new error will be shown to the user if it is the only one, or else it will wait in a
    /// queue until older errors have been acknowledged.
    pub(crate) fn add_error(&mut self, err: ShowError) {
        self.show_errors.push_front(err);
    }

    /// Add a tooltip to the GUI.
    ///
    /// The tooltip must be displayed until it expires or this will "leak" tooltips.
    fn add_tooltip(&mut self, tooltip_id: egui::Id, label: &str) {
        self.show_tooltips
            .insert(tooltip_id, (label.to_owned(), Instant::now()));
    }

    /// Show a tooltip at the current cursor position for the given duration.
    ///
    /// The tooltip must have already been added for it to be displayed.
    fn tooltip(
        &mut self,
        ctx: &egui::Context,
        ui: &egui::Ui,
        tooltip_id: egui::Id,
        duration: Duration,
    ) {
        if let Some((label, created)) = self.show_tooltips.remove(&tooltip_id) {
            if Instant::now().duration_since(created) < duration {
                let tooltip_position = ui.available_rect_before_wrap().min;
                egui::containers::popup::show_tooltip_at(
                    ctx,
                    tooltip_id,
                    Some(tooltip_position),
                    |ui| {
                        ui.label(&label);
                    },
                );

                // Put the tooltip back until it expires
                self.show_tooltips.insert(tooltip_id, (label, created));
            }
        }
    }
}

/// Holds state for an error message to show to the user, and provides a feedback mechanism for the
/// user to make a decision on how to handle the error.
pub(crate) struct ShowError {
    /// The error message.
    error: String,
    /// Simple description for the user
    context: String,
}

impl ShowError {
    /// Create an default error message to be shown to the user.
    ///
    ///
    pub(crate) fn new(err: String) -> Self {
        Self {
            error: err,
            context: "An error occurred".to_string(),
        }
    }

    /// Create an error message to be shown to the user. customize the context
    ///
    ///
    #[allow(dead_code)]
    pub(crate) fn new_custom_context(err: String, context: String) -> Self {
        Self {
            error: err,
            context,
        }
    }
}
