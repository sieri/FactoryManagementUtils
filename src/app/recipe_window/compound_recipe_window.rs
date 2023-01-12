use crate::app::commons::CommonsManager;
use crate::app::recipe_graph::RecipeGraph;

use crate::app::recipe_window::base_recipe_window::BaseRecipeWindow;
use crate::app::recipe_window::RecipeWindowGUI;
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::resource_flow::ManageResourceFlow;
use crate::app::resources::ManageFlow;
use egui::Context;
use std::fmt::Error;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct CompoundRecipeWindow {
    ///Graph of a recipe
    recipe_graph: RecipeGraph,

    ///base window managing the internals of the recipe window
    pub(crate) inner_recipe: BaseRecipeWindow,
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
            recipe_graph,
            inner_recipe: BaseRecipeWindow::new(title),
        };
        graph.update_interface();
        graph
    }
    fn update_interface(&mut self) {
        self.inner_recipe.inputs.clear();
        for source in self.recipe_graph.sources.iter() {
            self.inner_recipe
                .inputs
                .push(ManageFlow::RecipeInput(RecipeInputResource::new(
                    source.output.resource().clone(),
                    source.output.total_out(),
                )));
        }
        self.inner_recipe.outputs.clear();
        for sink in self.recipe_graph.sinks.iter() {
            if let Some(flow) = &sink.sink {
                self.inner_recipe
                    .outputs
                    .push(ManageFlow::RecipeInput(RecipeInputResource::new(
                        flow.resource().clone(),
                        flow.total_in(),
                    )));
            }
        }
    }
}

impl RecipeWindowGUI for CompoundRecipeWindow {
    fn show(&mut self, _commons: &mut CommonsManager, _ctx: &Context, _enabled: bool) -> bool {
        let mut _open = true;
        self.inner_recipe.clean_coordinates();

        todo!()
    }

    fn generate_tooltip(&self) -> Result<String, Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {}
