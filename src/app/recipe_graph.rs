use crate::app::commons::CommonsManager;
use crate::app::recipe_window::arrow_flow::ArrowFlow;
use crate::app::recipe_window::basic_recipe_window::BasicRecipeWindow;
use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;
use crate::app::recipe_window::resource_sink::ResourceSink;
use crate::app::recipe_window::resources_sources::ResourceSource;
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ManageFlow;
use crate::app::{FlowCalculatorHelper, FlowCalculatorType};
use egui::Context;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::fmt::Error;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RecipeGraph {
    pub recipes: Vec<BasicRecipeWindow>,
    pub compound_recipes: Vec<CompoundRecipeWindow>,
    pub sources: Vec<ResourceSource>,
    pub sinks: Vec<ResourceSink>,
    pub arrows: Vec<ArrowFlow>,
}

impl RecipeGraph {
    pub fn new() -> Self {
        Self {
            recipes: vec![],
            sources: vec![],
            sinks: vec![],
            arrows: vec![],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.recipes.clear();
        self.sources.clear();
        self.sinks.clear();
        self.arrows.clear();
    }

    pub fn calculate(&mut self) {
        println!("==================Calculate==================");

        let mut sources_helpers = LinkedList::new();
        let mut sinks_helpers = LinkedList::new();
        let mut recipes_helpers = vec![(0, LinkedList::new()); self.recipes.len()];
        //reset flows
        for source in self.sources.iter_mut() {
            source.output.reset();
        }

        for recipe in self.recipes.iter_mut() {
            for input in recipe.inputs.iter_mut() {
                match input {
                    ManageFlow::RecipeInput(r) => r.reset(),
                    ManageFlow::RecipeOutput(_) => {}
                }
            }
            for output in recipe.outputs.iter_mut() {
                match output {
                    ManageFlow::RecipeInput(_) => {}
                    ManageFlow::RecipeOutput(r) => r.reset(),
                }
            }
        }

        for sink in self.sinks.iter_mut() {
            if let Some(f) = sink.sink.as_mut() {
                f.reset();
            }
        }

        //build relationships from arrows
        for arrow in self.arrows.iter() {
            let start_id = arrow.start_flow_window;
            let source_flow_index = arrow.start_flow_index;
            let source_type = arrow.start_flow_type;

            let source_window_index = match source_type {
                RecipeWindowType::Basic => {
                    self.recipes.iter().position(|recipe| recipe.id == start_id)
                }
                RecipeWindowType::Source => {
                    self.sources.iter().position(|recipe| recipe.id == start_id)
                }
                RecipeWindowType::Sink => {
                    println!("incorrect source type");
                    None
                }
            };

            let end_id = arrow
                .end_flow_window
                .unwrap_or_else(|| egui::Id::new("Invalid ID"));
            let end_flow_index = arrow.end_flow_index;
            let end_type = arrow.end_flow_type.unwrap_or(RecipeWindowType::Source);
            let end_window_index = match end_type {
                RecipeWindowType::Basic => {
                    self.recipes.iter().position(|recipe| recipe.id == end_id)
                }
                RecipeWindowType::Source => None,
                RecipeWindowType::Sink => self.sinks.iter().position(|sink| sink.id == end_id),
            };

            if let Some(source_window_index) = source_window_index {
                if let Some(end_window_index) = end_window_index {
                    let helper = FlowCalculatorHelper {
                        source_window_index,
                        source_flow_index,
                        end_window_index,
                        end_flow_index,

                        source_type,
                        end_type,
                    };

                    match source_type {
                        RecipeWindowType::Basic => {
                            let source_order = recipes_helpers[source_window_index].0;
                            let mut end_order = recipes_helpers[end_window_index].0;

                            if source_order >= end_order {
                                end_order = source_order + 1usize;
                            }

                            recipes_helpers[end_window_index]
                                .1
                                .push_back(FlowCalculatorType::Helper(helper));
                            recipes_helpers[end_window_index].0 = end_order;
                        }
                        RecipeWindowType::Source => {
                            sources_helpers.push_back(FlowCalculatorType::Helper(helper));
                        }
                        RecipeWindowType::Sink => {
                            sinks_helpers.push_back(FlowCalculatorType::Helper(helper))
                        }
                    }
                }
            }
        }
        let mut calculate_helper = sources_helpers;

        recipes_helpers.sort_by(|helper1, helper2| helper1.0.partial_cmp(&helper2.0).unwrap());

        for (order, mut list) in recipes_helpers {
            println!("{order}");
            if list.front().is_some() {
                let index = match list.front().unwrap() {
                    FlowCalculatorType::Helper(h) => h.end_window_index,
                    FlowCalculatorType::EndRecipe(i) => *i,
                };
                list.push_back(FlowCalculatorType::EndRecipe(index));
                calculate_helper.append(&mut list);
            }
        }

        calculate_helper.append(&mut sinks_helpers);

        //calculate
        for calculate_helper in calculate_helper.iter_mut() {
            match calculate_helper {
                FlowCalculatorType::Helper(h) => match h.source_type {
                    RecipeWindowType::Basic => {
                        let source = &mut self.recipes[h.source_window_index];
                        let source_flow = &mut source.outputs[h.source_flow_index];
                        match source_flow {
                            ManageFlow::RecipeInput(_) => {
                                println!("Resource source wrong")
                            }
                            ManageFlow::RecipeOutput(o) => {
                                let used_flow = o.created.clone();
                                let added_source = o.add_out_flow(used_flow.clone());

                                let end_flow = match h.end_type {
                                    RecipeWindowType::Basic => {
                                        let end = &mut self.recipes[h.end_window_index];
                                        let f = match &mut end.inputs[h.end_flow_index] {
                                            ManageFlow::RecipeInput(r) => r,
                                            ManageFlow::RecipeOutput(_) => {
                                                panic!("Never happens")
                                            }
                                        };
                                        Some(f)
                                    }
                                    RecipeWindowType::Source => None,
                                    RecipeWindowType::Sink => {
                                        let end = &mut self.sinks[h.end_window_index];
                                        let f = end.sink.as_mut();
                                        match f {
                                            None => {
                                                let f = RecipeInputResource::new(
                                                    used_flow.resource.clone(),
                                                    used_flow.clone(),
                                                );
                                                end.sink = Some(f);
                                                let f = end.sink.as_mut().unwrap();
                                                Some(f)
                                            }
                                            Some(f) => Some(f),
                                        }
                                    }
                                }
                                .unwrap();

                                let added_input = end_flow.add_in_flow(used_flow);

                                if !(added_source && added_input) {
                                    println!("Error: added_source:{added_source} added_inputs{added_input}");
                                }
                            }
                        }
                    }
                    RecipeWindowType::Source => {
                        let source = &mut self.sources[h.source_window_index];
                        let end_flow = match h.end_type {
                            RecipeWindowType::Basic => {
                                let end = &mut self.recipes[h.end_window_index];
                                let f = match &mut end.inputs[h.end_flow_index] {
                                    ManageFlow::RecipeInput(r) => r,
                                    ManageFlow::RecipeOutput(_) => {
                                        panic!("Never happens")
                                    }
                                };
                                Some(f)
                            }
                            RecipeWindowType::Source => None,
                            RecipeWindowType::Sink => {
                                let end = &mut self.sinks[h.end_window_index];
                                let f = end.sink.as_mut().unwrap();
                                Some(f)
                            }
                        };
                        if let Some(end_flow) = end_flow {
                            if source.limited_output {
                                let used_flow = source.output.created.clone();
                                add_flows(&mut source.output, end_flow, used_flow);
                            } else {
                                let used_flow = end_flow.needed.clone();
                                add_flows(&mut source.output, end_flow, used_flow);
                            }
                        }
                    }
                    RecipeWindowType::Sink => {}
                },

                FlowCalculatorType::EndRecipe(_) => {}
            }
        }
    }
}

fn add_flows(
    source: &mut RecipeOutputResource<usize>,
    input: &mut RecipeInputResource<usize>,
    used_flow: ResourceFlow<usize, f32>,
) {
    let added_source = source.add_out_flow(used_flow.clone());
    let added_input = input.add_in_flow(used_flow);

    if !(added_source && added_input) {
        println!("Error: added_source:{added_source} added_inputs{added_input}");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::app::recipe_window::arrow_flow::ArrowFlow;
    use crate::app::recipe_window::basic_recipe_window::tests::setup_basic_recipe_one_to_one;
    use crate::app::recipe_window::resource_sink::ResourceSink;
    use crate::app::recipe_window::resources_sources::ResourceSource;
    use crate::app::recipe_window::RecipeWindowType;
    use crate::app::resources::resource_flow::test::setup_flow_resource_a;
    use eframe::epaint::ahash::HashMapExt;

    use crate::app::recipe_graph::RecipeGraph;
    use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
    use crate::app::resources::{RatePer, ResourceDefinition};
    use crate::test_framework as t;
    use crate::test_framework::TestResult;
    use crate::FactoryManagementApp;
    use egui::epaint::ahash::HashMap;
    use egui::{LayerId, Order};

    pub(crate) struct TestInfo {
        pub name: String,
        pub graph: RecipeGraph,
        pub inputs: Vec<ResourceFlow<usize, f32>>,
        pub outputs: Vec<ResourceFlow<usize, f32>>,
    }

    impl RecipeGraph {
        pub(crate) fn setup_empty_graph() -> TestInfo {
            TestInfo {
                name: "empty graph".to_string(),
                graph: RecipeGraph::new(),
                inputs: vec![],
                outputs: vec![],
            }
        }

        pub(crate) fn setup_simple_graph() -> TestInfo {
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };

            let mut graph = RecipeGraph::new();
            let recipe_1t1 = setup_basic_recipe_one_to_one();
            let sink = ResourceSink::new();
            let resource_a_flow = setup_flow_resource_a();
            let source = ResourceSource::new(resource_a_flow.resource.name.clone());
            let mut first_arrow = ArrowFlow::new(
                recipe_1t1.output_resource.first().unwrap().def.clone(),
                recipe_1t1.recipe.id,
                RecipeWindowType::Basic,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(recipe_1t1.output_resource.first().unwrap().def.clone()),
                    sink.id,
                    RecipeWindowType::Sink,
                    0,
                )
                .expect("arrow error");
            let mut second_arrow = ArrowFlow::new(
                resource_a_flow.resource.clone(),
                source.id,
                RecipeWindowType::Source,
                dummy_layer,
                0,
            );
            second_arrow
                .put_end(
                    Some(resource_a_flow.resource),
                    recipe_1t1.recipe.id,
                    RecipeWindowType::Basic,
                    0,
                )
                .expect("arrow error");
            graph.recipes.push(recipe_1t1.recipe);
            graph.arrows.push(first_arrow);
            graph.arrows.push(second_arrow);
            graph.sources.push(source);
            graph.sinks.push(sink);
            let input = recipe_1t1.input_resource.first().unwrap();
            let output = recipe_1t1.output_resource.first().unwrap();
            TestInfo {
                name: "one to one".to_string(),
                graph,
                inputs: vec![ResourceFlow::new(
                    &input.def.clone(),
                    input.amount_per_cycle,
                    input.amount,
                    input.rate,
                )],
                outputs: vec![ResourceFlow::new(
                    &output.def.clone(),
                    output.amount_per_cycle,
                    output.amount,
                    output.rate,
                )],
            }
        }

        fn get_calc_sources(&self) -> HashMap<ResourceDefinition, (f32, RatePer)> {
            let mut result = HashMap::new();
            for source in self.sources.iter() {
                let flow = source.output.total_out();
                result.insert(flow.resource, (flow.amount, flow.rate));
            }

            result
        }

        fn get_calc_sinks(&self) -> HashMap<ResourceDefinition, (f32, RatePer)> {
            let mut result = HashMap::new();
            for sink in self.sinks.iter() {
                let flow = sink.sink.as_ref().unwrap().total_in();
                result.insert(flow.resource, (flow.amount, flow.rate));
            }

            result
        }
    }
    // ------------------------------- Test -------------------------------

    #[test]
    fn test_calculation() -> TestResult {
        let test_infos = setup_test_graphs();

        for test_info in test_infos {
            println!("Start test on graph: {}", test_info.name);
            let mut app = test_info.graph;
            app.calculate();
            let calculated_inputs = app.get_calc_sources();
            println!("Calculated: {calculated_inputs:?}");
            for input in test_info.inputs.iter() {
                let resource = input.resource.clone();
                println!("Resource{resource}");
                let calculated = calculated_inputs
                    .get(&resource)
                    .expect("no data in the resource");
                assert_eq!(
                    input.amount, calculated.0,
                    "Amount of an input doesn't match",
                );
                assert_eq!(input.rate, calculated.1, "Rate of an input doesn't match");
            }
            let calculated_outputs = app.get_calc_sinks();
            for output in test_info.outputs.iter() {
                let resource = output.resource.clone();
                let calculated = calculated_outputs
                    .get(&resource)
                    .expect("no data in the resource");
                assert_eq!(
                    output.amount, calculated.0,
                    "Amount of an output doesn't match",
                );
                assert_eq!(output.rate, calculated.1, "Rate of an output doesn't match");
            }
        }

        Ok(())
    }

    fn setup_test_graphs() -> [TestInfo; 2] {
        [
            RecipeGraph::setup_empty_graph(),
            RecipeGraph::setup_simple_graph(),
        ]
    }
}
