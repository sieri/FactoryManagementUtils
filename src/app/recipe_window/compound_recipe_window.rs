use crate::app::commons::CommonsManager;
use crate::app::recipe_graph::RecipeGraph;

use crate::app::recipe_window::base_recipe_window::{BaseRecipeWindow, ConfigFeatures};
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::ManageResourceFlow;
use crate::app::resources::ManageFlow;
use egui::Context;
use std::fmt::Error;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
            inner_recipe: BaseRecipeWindow::new(
                title,
                ConfigFeatures {
                    interactive_input: false,
                    pure_time_input: true,
                    interactive_output: false,
                    pure_time_output: true,
                    show_power: false,
                    show_time: false,
                },
                RecipeWindowType::CompoundRecipe,
            ),
        };
        graph.update_interface();
        graph
    }
    fn update_interface(&mut self) {
        self.recipe_graph.calculate();
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
                self.inner_recipe.outputs.push(ManageFlow::RecipeOutput(
                    RecipeOutputResource::new(flow.resource().clone(), flow.total_in()),
                ));
            }
        }
    }
}

impl RecipeWindowGUI for CompoundRecipeWindow {
    fn show(&mut self, commons: &mut CommonsManager, ctx: &Context, enabled: bool) -> bool {
        let mut open = true;
        self.inner_recipe.clean_coordinates();
        let title = self.inner_recipe.get_title();
        let response = self
            .inner_recipe
            .window(commons, ctx, enabled, &mut open, title);
        let inner_response = response.unwrap();
        self.inner_recipe.update_coordinates(&inner_response);
        self.inner_recipe
            .show_tooltips(commons, ctx, inner_response);

        self.inner_recipe.push_errors(commons);

        self.inner_recipe.push_coordinates(commons, &mut open);

        open
    }

    fn generate_tooltip(&self) -> Result<String, Error> {
        todo!()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::app::recipe_graph;
    use crate::app::recipe_graph::RecipeGraph;
    use crate::app::recipe_window::base_recipe_window::tests::RecipeResourceInfos;
    use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;

    use crate::app::resources::ManageFlow;
    use crate::app::resources::ManageFlow::RecipeInput;

    pub(crate) struct TestInfo {
        pub recipe: CompoundRecipeWindow,
        pub input_resources: Vec<RecipeResourceInfos>,
        pub output_resources: Vec<RecipeResourceInfos>,
        pub graph_info: recipe_graph::tests::TestInfo,
    }

    impl CompoundRecipeWindow {
        pub(crate) fn setup_from_graph_info(graph_info: recipe_graph::tests::TestInfo) -> TestInfo {
            let recipe = Self::new(graph_info.graph.clone());
            let mut input_resources = vec![];
            let mut output_resources = vec![];
            for input in graph_info.inputs.iter() {
                input_resources.push(input.into())
            }
            for output in graph_info.outputs.iter() {
                output_resources.push(output.into())
            }
            TestInfo {
                recipe,
                input_resources,
                output_resources,
                graph_info,
            }
        }

        pub(crate) fn setup_one_to_one_compound() -> TestInfo {
            Self::setup_from_graph_info(RecipeGraph::setup_simple_graph())
        }

        pub(crate) fn setup_one_to_one_compound_two_levels() -> TestInfo {
            Self::setup_from_graph_info(RecipeGraph::setup_simple_compound_graph())
        }
    }

    pub(crate) fn set_list_of_compounds_windows() -> [TestInfo; 2] {
        [
            CompoundRecipeWindow::setup_one_to_one_compound(),
            CompoundRecipeWindow::setup_one_to_one_compound_two_levels(),
        ]
    }

    pub(crate) fn check_flow_and_test_info(
        result: &ManageFlow<usize>,
        expected: &RecipeResourceInfos,
    ) {
        let flow = match result {
            RecipeInput(r) => &r.needed,
            ManageFlow::RecipeOutput(r) => &r.created,
        };

        assert_eq!(expected.def, flow.resource, "Resource not the expected one");
        assert_eq!(expected.amount, flow.amount, "amounts doesn't match");
        assert_eq!(expected.rate, flow.rate, "rates doesn't match");
    }

    //----------------------------------Tests------------------------------------------

    #[test]
    fn test_compound_windows() {
        println!("====================Testing Compound Windows====================");
        let test_infos = set_list_of_compounds_windows();

        for test_info in test_infos.iter() {
            let recipe = &test_info.recipe;
            for (result, expected) in recipe
                .inner_recipe
                .inputs
                .iter()
                .zip(test_info.input_resources.iter())
            {
                check_flow_and_test_info(result, expected);
            }
            for (result, expected) in recipe
                .inner_recipe
                .outputs
                .iter()
                .zip(test_info.output_resources.iter())
            {
                check_flow_and_test_info(result, expected);
            }
        }
    }
}
