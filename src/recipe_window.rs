use crate::resources::AnyManageResourceFlow;
use eframe::emath::Align;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait RecipeWindowGUI {
    /// Show a recipe window on the frame
    ///
    /// # Arguments
    ///
    /// * `ctx`: the context it will spawn on
    /// * `enabled`: flag indicating it's enabled
    ///
    /// returns: `bool` flag if the window is still alive
    ///
    fn show(&self, ctx: &egui::Context, enabled: bool) -> bool;
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Descriptor for a Basic Recipe window, the recipe is directly calculated
pub struct BasicRecipeWindowDescriptor {
    ///Title of the recipe
    title: String,

    ///unique id of the recipe
    id: egui::Id,

    ///list of inputs
    #[serde(skip)] //TODO: Serialize this probably manually
    inputs: Vec<Box<dyn AnyManageResourceFlow>>,

    ///list of outputs
    #[serde(skip)] //TODO: Serialize this probably manually
    outputs: Vec<Box<dyn AnyManageResourceFlow>>,

    ///power used per cycle
    #[serde(skip)] //TODO: Serialize this probably manually
    power: Option<Box<dyn AnyManageResourceFlow>>,
}

impl Default for BasicRecipeWindowDescriptor {
    fn default() -> Self {
        Self::new(String::from("Basic Recipe Window"))
    }
}

impl RecipeWindowGUI for BasicRecipeWindowDescriptor {
    fn show(&self, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                        self.show_inputs(ui, enabled);
                        self.show_outputs(ui, enabled);
                    });
                })
            });

        open
    }
}

impl BasicRecipeWindowDescriptor {
    /// Create a new basic recipe
    ///
    /// # Arguments
    ///
    /// * `title`: Title of the recipe
    ///
    /// returns: BasicRecipeWindowDescriptor
    pub fn new(title: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = egui::Id::new(title.to_owned() + &*format!("{}", timestamp));
        Self {
            title,
            id,
            inputs: vec![],
            outputs: vec![],
            power: None,
        }
    }

    fn show_inputs(&self, ui: &mut egui::Ui, enabled: bool) {
        let _ = ui.label("inputs");

        if ui
            .add_enabled(
                enabled,
                egui::Button::new(egui::RichText::new("➕").color(egui::Rgba::GREEN)),
            )
            .clicked()
        {}
    }
    fn show_outputs(&self, ui: &mut egui::Ui, enabled: bool) {
        let _ = ui.label("outputs");

        if ui
            .add_enabled(
                enabled,
                egui::Button::new(egui::RichText::new("➕").color(egui::Rgba::GREEN)),
            )
            .clicked()
        {}
    }
}
