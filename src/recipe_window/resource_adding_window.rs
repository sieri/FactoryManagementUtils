use crate::app::CommonManager;
use crate::recipe_window::RecipeWindowGUI;
use crate::resources::ManageFlow::{RecipeInput, RecipeOutput};
use crate::resources::{
    ManageFlow, RatePer, RecipeInputResource, RecipeOutputResource, ResourceDefinition,
    ResourceFlow, Unit,
};
use crate::utils::{Io, Number};
use egui::Widget;
use num_traits::One;
use std::fmt::Write;
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResourceAddingWindow<T> {
    ///Title
    pub(crate) title: String,

    ///private id
    id: egui::Id,

    ///Resource Name
    pub(crate) resource_name: String,

    ///Amount per cycle
    pub(crate) amount_per_cycle: T,

    ///Amount per unit of time
    pub(crate) amount_per_time: T,

    ///Rate
    pub(crate) rate: RatePer,

    ///Io direction
    pub(crate) dir: Io,

    ///flag indicating if validated
    pub(crate) okay: bool,
}

impl<T: Number> ResourceAddingWindow<T> {
    pub fn new(title: String, dir: Io) -> Self {
        Self {
            title: title.clone(),
            id: Self::gen_id(title),
            resource_name: "".to_string(),
            amount_per_cycle: T::one(),
            amount_per_time: T::one(),
            rate: RatePer::Second,
            dir,
            okay: false,
        }
    }

    pub(crate) fn get_resource(&self) -> ManageFlow<T> {
        let resource = ResourceDefinition {
            name: self.resource_name.clone(),
            unit: Unit::Piece,
        };
        let flow = ResourceFlow::new(&resource, self.amount_per_cycle, 1.0f32, RatePer::Second);
        match self.dir {
            Io::Input => RecipeInput(RecipeInputResource::new(resource, flow)),
            Io::Output => RecipeOutput(RecipeOutputResource::new(resource, flow)),
        }
    }
}

impl<T: Number> RecipeWindowGUI for ResourceAddingWindow<T> {
    fn show(&mut self, _commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;

        let response = egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if egui::TextEdit::singleline(&mut self.resource_name)
                            .hint_text("resource name")
                            .show(ui)
                            .response
                            .has_focus()
                        {
                            let input = ctx.input();
                            if input.key_pressed(egui::Key::Enter) {
                                self.okay = true;
                            }
                        };
                        ui.label("Amount:");

                        //ui.horizontal( |ui| {
                        egui::DragValue::new(&mut self.amount_per_cycle).ui(ui);
                        ui.label("per cycle");
                        // });
                        //TODO Implement time based entering
                        //egui::DragValue::new(&mut self.amount_per_time).ui(ui);
                    });
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let button = ui.button(match self.dir {
                            Io::Input => "Add input",
                            Io::Output => "Add output",
                        });
                        if button.clicked() {
                            self.okay = true;
                        }
                    });
                });
            });

        if let Some(inner) = response {
            if inner.response.has_focus() {
                let input = ctx.input();
                if input.key_pressed(egui::Key::Enter) {
                    self.okay = true;
                }
            }
        }
        open
    }

    fn generate_tooltip(&self) -> Result<String, std::fmt::Error> {
        Ok("Window to add resources".to_string())
    }
}
