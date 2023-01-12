use crate::app::commons::CommonsManager;
use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::error::ShowError;
use crate::app::recipe_graph::RecipeGraph;
use crate::app::recipe_window;
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::resource_flow::ManageResourceFlow;
use crate::app::resources::ManageFlow;
use crate::app::resources::ManageFlow::{RecipeInput, RecipeOutput};
use crate::utils::Io;
use egui::{Context, Widget};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct CompoundRecipeWindow {
    ///Title of the recipe
    pub(crate) title: String,

    ///ID
    pub(crate) id: egui::Id,

    ///Graph of a recipe
    recipe_graph: RecipeGraph,

    ///list of inputs
    pub(crate) inputs: Vec<ManageFlow<usize>>,

    ///list of outputs
    pub(crate) outputs: Vec<ManageFlow<usize>>,

    ///Flag indicating if every inputs have sufficient resources
    stable_in: bool,

    ///Flag indicating if every outputs have sufficient draining
    stable_out: bool,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,

    #[serde(skip)]
    errors: Vec<ShowError>,
}

impl CompoundRecipeWindow {
    pub fn new(recipe_graph: RecipeGraph) -> Self {
        let title = if let Some(first) = recipe_graph.sinks.first() {
            if let Some(recipe_resource) = &first.sink {
                recipe_resource.needed.resource.name.clone()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let mut graph = Self {
            title: format!("Compound recipe: {}", title),
            id: Self::gen_id(title),
            recipe_graph,
            inputs: vec![],
            outputs: vec![],
            stable_in: false,
            stable_out: false,
        };
        graph.update_interface();
        graph
    }
    fn update_interface(&mut self) {
        self.inputs.clear();
        for source in self.recipe_graph.sources.iter() {
            self.inputs
                .push(ManageFlow::RecipeInput(RecipeInputResource::new(
                    source.output.resource().clone(),
                    source.output.total_out(),
                )));
        }
        self.outputs.clear();
        for sink in self.recipe_graph.sinks.iter() {
            if let Some(flow) = &sink.sink {
                self.outputs
                    .push(ManageFlow::RecipeInput(RecipeInputResource::new(
                        flow.resource().clone(),
                        flow.total_in(),
                    )));
            }
        }
    }

    fn show_inputs(&mut self, commons: &mut CommonsManager, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Inputs");
            });

            for i in 0..self.inputs.len() {
                self.show_flow(commons, i, Io::Input, ui, enabled);
            }
        });
    }

    fn show_outputs(&mut self, commons: &mut CommonsManager, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Outputs");
            });
            for i in 0..self.outputs.len() {
                self.show_flow(commons, i, Io::Output, ui, enabled);
            }
        });
    }

    fn show_flow(
        &mut self,
        commons: &mut CommonsManager,
        resource_flow_index: usize,
        dir: Io,
        ui: &mut egui::Ui,
        _enabled: bool,
    ) {
        let mut changed = false;
        //get variables
        let resource_flow: &mut dyn ManageResourceFlow<usize> = match dir {
            Io::Input => match &mut self.inputs[resource_flow_index] {
                RecipeInput(r) => r,
                RecipeOutput(_) => {
                    panic!("Error!!! Impossible situation")
                }
            },
            Io::Output => match &mut self.outputs[resource_flow_index] {
                RecipeInput(_) => {
                    panic!("Error!!! Impossible situation")
                }
                RecipeOutput(r) => r,
            },
        };
        let resource = resource_flow.resource();

        let mut name = resource.clone().name;

        self.stable_in &= resource_flow.is_enough();

        let color = match resource_flow.is_enough() {
            true => egui::Rgba::GREEN,
            false => egui::Rgba::RED,
        };

        let flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let rate = flow.rate;
        let mut amount = flow.amount_per_cycle;
        let mut amount_per_time = flow.amount;

        ui.horizontal(|ui| {
            match dir {
                Io::Input => {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.in_flow.push(btn_resp.rect);

                    if btn_resp.clicked() && commons.arrow_active {
                        commons.clicked_place_arrow_info = Some((
                            Some(resource.clone()),
                            self.id,
                            resource_flow_index,
                            RecipeWindowType::Compound,
                        ));
                    }
                }
                Io::Output => {}
            }

            recipe_window::text_edit(ui, &mut name);
            ui.label(":");

            ui.horizontal(|ui| {
                changed |= egui::DragValue::new(&mut amount_per_time).ui(ui).changed();
                ui.label(egui::RichText::new(rate.to_shortened_string()).color(color))
            });

            match dir {
                Io::Input => {}
                Io::Output => {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.out_flow.push(btn_resp.rect);

                    if btn_resp.clicked() {
                        commons.clicked_start_arrow_info = Some((
                            resource.clone(),
                            self.id,
                            ui.layer_id(),
                            resource_flow_index,
                            RecipeWindowType::Basic,
                        ));
                    }
                }
            }
        });

        if changed {
            if amount != flow.amount_per_cycle {
                println!("Hello!");
                match dir {
                    Io::Input => {
                        resource_flow.set_designed_amount_per_cycle(amount);
                    }
                    Io::Output => {
                        resource_flow.set_designed_amount_per_cycle(amount);
                    }
                }
            }

            commons.recalculate = true;
            let r = self.update_flow(dir);
            if let Err(e) = r {
                commons.add_error(ShowError::new(e.to_string()));
            }
        }

        commons.recalculate |= changed;
    }
}

impl RecipeWindowGUI for CompoundRecipeWindow {
    fn show(&mut self, commons: &mut CommonsManager, ctx: &Context, enabled: bool) -> bool {
        todo!()
    }

    fn generate_tooltip(&self) -> Result<String, Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {}
