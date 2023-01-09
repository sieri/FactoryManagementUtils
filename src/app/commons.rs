pub mod recipes_lists;

use crate::app::commons::recipes_lists::SavedRecipes;
use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::error::ShowError;
use crate::app::recipe_window::basic_recipe_window_descriptor::BasicRecipeWindowDescriptor;
use crate::app::recipe_window::RecipeWindowType;
use crate::app::resources::ResourceDefinition;
use egui::Context;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CommonsManager {
    #[serde(skip)]
    pub window_coordinates: HashMap<egui::Id, CoordinatesInfo>,
    #[serde(skip)]
    pub arrow_active: bool,
    #[serde(skip)]
    pub clicked_start_arrow_info: Option<(
        ResourceDefinition,
        egui::Id,
        egui::LayerId,
        usize,
        RecipeWindowType,
    )>,
    #[serde(skip)]
    pub clicked_place_arrow_info: Option<(
        Option<ResourceDefinition>,
        egui::Id,
        usize,
        RecipeWindowType,
    )>,
    #[serde(skip)]
    pub recalculate: bool,

    /// List of error popups to keep
    #[serde(skip)]
    pub show_errors: VecDeque<ShowError>,

    /// List of tooltips that can be shown
    #[serde(skip)]
    pub show_tooltips: HashMap<egui::Id, (String, Instant)>,

    ///list of saved recipes
    pub saved_recipes: SavedRecipes,
}

impl CommonsManager {
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
    pub(crate) fn add_tooltip(&mut self, tooltip_id: egui::Id, label: String) {
        self.show_tooltips
            .insert(tooltip_id, (label, Instant::now()));
    }

    pub(crate) fn has_tooltip(&self, tooltip_id: egui::Id) -> bool {
        self.show_tooltips.contains_key(&tooltip_id)
    }

    /// Show a tooltip at the current cursor position for the given duration.
    ///
    /// The tooltip must have already been added for it to be displayed.
    pub(crate) fn tooltip(
        &mut self,
        ctx: &Context,
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

    ///Save recipes
    pub fn save(&mut self, recipe: &mut BasicRecipeWindowDescriptor) {
        let title = recipe.get_title();
        let data = recipe.save();
        if let Some(data) = data {
            self.saved_recipes.push(title, data);
        }
    }
}

impl Default for CommonsManager {
    fn default() -> Self {
        CommonsManager {
            window_coordinates: Default::default(),
            arrow_active: false,
            clicked_start_arrow_info: None,
            clicked_place_arrow_info: None,
            recalculate: false,
            show_errors: Default::default(),
            show_tooltips: Default::default(),
            saved_recipes: Default::default(),
        }
    }
}
