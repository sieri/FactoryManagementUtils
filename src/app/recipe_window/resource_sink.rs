use crate::app::commons::CommonsManager;
use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::recipe_window;
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::resource_flow::ManageResourceFlow;
use crate::app::resources::RatePer;
use egui::Widget;
use std::fmt::Write;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct ResourceSink {
    ///unique id of the resource
    pub(crate) id: egui::Id,

    ///output
    pub(crate) sink: Option<RecipeInputResource<usize>>,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,
}

impl ResourceSink {
    pub(crate) fn new() -> Self {
        Self {
            id: Self::gen_id("ResourceSink".to_string()),
            sink: None,
            window_coordinate: Default::default(),
        }
    }
}

impl RecipeWindowGUI for ResourceSink {
    fn show(&mut self, commons: &mut CommonsManager, ctx: &egui::Context, enabled: bool) -> bool {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();

        let mut open = true;

        let response = egui::Window::new("Resource sink")
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.in_flow.push(btn_resp.rect);

                    if btn_resp.clicked() && commons.arrow_active {
                        commons.clicked_place_arrow_info = Some((
                            self.sink.as_ref().map(|sink| sink.resource()),
                            self.id,
                            0,
                            RecipeWindowType::Sink,
                        ));
                    }

                    if let Some(sink) = &self.sink {
                        let mut resource_name = sink.resource().name;
                        recipe_window::text_edit(ui, &mut resource_name);
                        let mut amount_per_time = sink.total_in().amount;
                        let rate = sink.total_in().rate;
                        egui::DragValue::new(&mut amount_per_time).ui(ui);
                        ui.label(egui::RichText::new(match rate {
                            RatePer::Tick => "/tick",
                            RatePer::Second => "/s",
                            RatePer::Minute => "/min ",
                            RatePer::Hour => "/h",
                        }));
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

        match &self.sink {
            None => {}
            Some(input) => {
                let flow = &input.needed;
                write!(
                    tooltip,
                    "Sink of {}. {} {}",
                    flow.resource.name, flow.amount, flow.rate
                )?;
            }
        }

        Ok(tooltip)
    }
}
