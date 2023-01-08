use crate::app::CommonManager;

use crate::resources::RatePer;

use std::f32;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) mod arrow_flow;
pub(crate) mod basic_recipe_window_descriptor;
pub(crate) mod resource_adding_window;
pub(crate) mod resource_sink;
pub(crate) mod resources_sources;

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
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool;

    fn gen_id(name: String) -> egui::Id {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        egui::Id::new(name + &*format!("{timestamp}"))
    }

    fn generate_tooltip(&self) -> Result<String, std::fmt::Error>;
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
pub enum RecipeWindowType {
    Basic,
    Source,
    Sink,
}

fn rate_combo(ui: &mut egui::Ui, rate: &mut RatePer) {
    egui::ComboBox::from_label("Time unit")
        .selected_text(format!("{rate:?}"))
        .show_ui(ui, |ui| {
            ui.selectable_value(rate, RatePer::Tick, "Tick");
            ui.selectable_value(rate, RatePer::Second, "Second");
            ui.selectable_value(rate, RatePer::Minute, "Minute");
            ui.selectable_value(rate, RatePer::Hour, "Hour");
        });
}

fn text_edit(ui: &mut egui::Ui, text: &mut String) {
    let text_len = text.len();
    egui::TextEdit::singleline(text)
        .desired_width((text_len * 7) as f32)
        .show(ui);
}
