use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::error::ShowError;
use crate::app::recipe_window::RecipeWindowType;
use crate::app::resources::ResourceDefinition;
use egui::Context;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub struct CommonsManager {
    pub window_coordinates: HashMap<egui::Id, CoordinatesInfo>,

    pub arrow_active: bool,

    pub clicked_start_arrow_info: Option<(
        ResourceDefinition,
        egui::Id,
        egui::LayerId,
        usize,
        RecipeWindowType,
    )>,
    pub clicked_place_arrow_info: Option<(
        Option<ResourceDefinition>,
        egui::Id,
        usize,
        RecipeWindowType,
    )>,

    pub recalculate: bool,

    // List of error popups to keep
    pub(crate) show_errors: VecDeque<ShowError>,

    /// List of tooltips that can be shown
    pub(crate) show_tooltips: HashMap<egui::Id, (String, Instant)>,
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
}
