use crate::app::commons::CommonsManager;
use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::recipe_window;
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::{RatePer, ResourceDefinition, Unit};
use egui::Widget;
use std::fmt::Write;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResourceSource {
    ///unique id of the resource
    pub(crate) id: egui::Id,

    ///output
    pub(crate) output: RecipeOutputResource<usize>,

    ///limited output
    pub(crate) limited_output: bool,

    ///limit amount
    limit_amount: usize,

    ///limit rate
    limit_rate: RatePer,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,
}

impl RecipeWindowGUI for ResourceSource {
    fn show(&mut self, commons: &mut CommonsManager, ctx: &egui::Context, enabled: bool) -> bool {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();

        let mut open = true;

        let resource = self.output.resource();

        let mut resource_name = resource.name.clone();

        let response = egui::Window::new("Resource source")
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    recipe_window::text_edit(ui, &mut resource_name);

                    ui.checkbox(&mut self.limited_output, "Limited");

                    if self.limited_output {
                        egui::DragValue::new(&mut self.limit_amount).ui(ui);
                        recipe_window::rate_combo(ui, &mut self.limit_rate);
                    }

                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.out_flow.push(btn_resp.rect);

                    if btn_resp.clicked() {
                        commons.clicked_start_arrow_info = Some((
                            resource.clone(),
                            self.id,
                            ui.layer_id(),
                            0,
                            RecipeWindowType::Source,
                        ));
                    }
                });
            });
        let inner_response = response.unwrap();
        self.window_coordinate.window = inner_response.response.rect;

        if inner_response.inner.is_none() {
            inner_response.response.on_hover_ui(|ui| {
                ui.label(
                    self.generate_tooltip()
                        .unwrap_or_else(|_| "Error generating tooltip".to_string()),
                );
            });
        }

        if open {
            commons
                .window_coordinates
                .insert(self.id, self.window_coordinate.clone());
        } else {
            commons.window_coordinates.remove(&self.id);
        }

        open
    }

    fn generate_tooltip(&self) -> Result<String, std::fmt::Error> {
        let mut tooltip = String::new();
        let flow = &self.output.created;
        write!(
            tooltip,
            "Source of {}. {} {}",
            flow.resource.name, flow.amount, flow.rate
        )?;
        Ok(tooltip)
    }
}

impl ResourceSource {
    pub fn new(resource: String) -> Self {
        let r = ResourceDefinition {
            name: resource.clone(),
            unit: Unit::Piece,
        };
        Self {
            id: Self::gen_id(resource),
            output: RecipeOutputResource::new(
                r.clone(),
                ResourceFlow::new(&r, 10, 1.0f32, RatePer::Tick),
            ),
            limited_output: false,
            limit_amount: 1,
            limit_rate: RatePer::Second,
            window_coordinate: Default::default(),
        }
    }
}
