use crate::app::recipe_window::arrow_flow::ArrowFlow;
use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;
use crate::app::recipe_window::resource_sink::ResourceSink;
use crate::app::recipe_window::resources_sources::ResourceSource;
use crate::app::recipe_window::simple_recipe_window::SimpleRecipeWindow;
use crate::app::recipe_window::RecipeWindowType;
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ManageFlow;
use crate::app::{FlowCalculatorHelper, FlowCalculatorType};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::mem::transmute;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RecipeGraph {
    pub simple_recipes: Vec<SimpleRecipeWindow>,
    pub compound_recipes: Vec<CompoundRecipeWindow>,
    pub sources: Vec<ResourceSource>,
    pub sinks: Vec<ResourceSink>,
    pub arrows: Vec<ArrowFlow>,
}

impl RecipeGraph {
    pub fn new() -> Self {
        Self {
            simple_recipes: vec![],
            compound_recipes: vec![],
            sources: vec![],
            sinks: vec![],
            arrows: vec![],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.simple_recipes.clear();
        self.sources.clear();
        self.sinks.clear();
        self.arrows.clear();
    }

    pub fn calculate(&mut self) {
        println!("==================Calculate==================");

        let mut sources_helpers = LinkedList::new();
        let mut sinks_helpers = LinkedList::new();

        let mut simple_recipes_helpers = vec![(0, LinkedList::new()); self.simple_recipes.len()];
        let mut compound_recipes_helpers =
            vec![(0, LinkedList::new()); self.compound_recipes.len()];

        self.reset_flows();
        let arrows = self.arrows.clone();
        //build relationships from arrows
        for arrow in arrows.iter() {
            let (start_flow_index, start_type, start_window_index) = self.get_startpoint(arrow);

            let (end_flow_index, end_type, end_window_index) = self.get_endpoint(arrow);

            if let Some(start_window_index) = start_window_index {
                if let Some(end_window_index) = end_window_index {
                    let helper = FlowCalculatorHelper {
                        start_window_index,
                        start_flow_index,
                        start_type,
                        end_window_index,
                        end_flow_index,
                        end_type,
                    };

                    match start_type {
                        RecipeWindowType::SimpleRecipe => {
                            let source_order = simple_recipes_helpers[start_window_index].0;
                            Self::connect_resources_helpers_ends(
                                &mut simple_recipes_helpers,
                                &mut compound_recipes_helpers,
                                &mut sinks_helpers,
                                end_type,
                                end_window_index,
                                helper,
                                source_order,
                            );
                        }
                        RecipeWindowType::Source => {
                            sources_helpers.push_back(FlowCalculatorType::Helper(helper));
                        }
                        RecipeWindowType::Sink => {
                            println!("Starting at a sink!!")
                        }
                        RecipeWindowType::CompoundRecipe => {
                            let source_order = compound_recipes_helpers[start_window_index].0;
                            Self::connect_resources_helpers_ends(
                                &mut simple_recipes_helpers,
                                &mut compound_recipes_helpers,
                                &mut sinks_helpers,
                                end_type,
                                end_window_index,
                                helper,
                                source_order,
                            );
                        }
                    }
                }
            }
        }

        let mut calculate_helper =
            Self::concatenate_helpers(sources_helpers, sinks_helpers, simple_recipes_helpers);
        //calculate
        self.perform_calculation(&mut calculate_helper)
    }

    fn connect_resources_helpers_ends(
        simple_recipes_helpers: &mut Vec<(usize, LinkedList<FlowCalculatorType>)>,
        compound_recipes_helpers: &mut Vec<(usize, LinkedList<FlowCalculatorType>)>,
        sinks_helpers: &mut LinkedList<FlowCalculatorType>,
        end_type: RecipeWindowType,
        end_window_index: usize,
        helper: FlowCalculatorHelper,
        source_order: usize,
    ) {
        match end_type {
            RecipeWindowType::SimpleRecipe => {
                let mut end_order = simple_recipes_helpers[end_window_index].0;

                //if the source is higher than end
                if source_order >= end_order {
                    //change the end order
                    end_order = source_order + 1usize;
                }

                //add the helper to the end point
                simple_recipes_helpers[end_window_index]
                    .1
                    .push_back(FlowCalculatorType::Helper(helper));
                //add order to the list
                simple_recipes_helpers[end_window_index].0 = end_order;
            }
            RecipeWindowType::CompoundRecipe => {
                let mut end_order: usize = compound_recipes_helpers[end_window_index].0;

                if source_order >= end_order {
                    end_order = source_order + 1usize;
                }

                compound_recipes_helpers[end_window_index]
                    .1
                    .push_back(FlowCalculatorType::Helper(helper));
                compound_recipes_helpers[end_window_index].0 = end_order;
            }
            RecipeWindowType::Source => {
                println!("Can't be done, ending at a sink")
            }
            RecipeWindowType::Sink => sinks_helpers.push_back(FlowCalculatorType::Helper(helper)),
        }
    }

    fn get_endpoint(&mut self, arrow: &ArrowFlow) -> (usize, RecipeWindowType, Option<usize>) {
        let end_id = arrow
            .end_flow_window
            .unwrap_or_else(|| egui::Id::new("Invalid ID"));
        let end_flow_index = arrow.end_flow_index;
        let end_type = arrow.end_flow_type.unwrap_or(RecipeWindowType::Source);
        let end_window_index = match end_type {
            RecipeWindowType::SimpleRecipe => self
                .simple_recipes
                .iter()
                .position(|recipe| recipe.inner_recipe.id == end_id),
            RecipeWindowType::Source => None,
            RecipeWindowType::Sink => self.sinks.iter().position(|sink| sink.id == end_id),
            RecipeWindowType::CompoundRecipe => self
                .compound_recipes
                .iter()
                .position(|recipe| recipe.inner_recipe.id == end_id),
        };
        (end_flow_index, end_type, end_window_index)
    }

    fn get_startpoint(&mut self, arrow: &ArrowFlow) -> (usize, RecipeWindowType, Option<usize>) {
        let start_id = arrow.start_flow_window;
        let source_flow_index = arrow.start_flow_index;
        let source_type = arrow.start_flow_type;

        let source_window_index = match source_type {
            RecipeWindowType::SimpleRecipe => self
                .simple_recipes
                .iter()
                .position(|recipe| recipe.inner_recipe.id == start_id),
            RecipeWindowType::Source => {
                self.sources.iter().position(|recipe| recipe.id == start_id)
            }
            RecipeWindowType::Sink => {
                println!("incorrect source type");
                None
            }
            RecipeWindowType::CompoundRecipe => self
                .compound_recipes
                .iter()
                .position(|recipe| recipe.inner_recipe.id == start_id),
        };
        (source_flow_index, source_type, source_window_index)
    }

    fn concatenate_helpers(
        sources_helpers: LinkedList<FlowCalculatorType>,
        mut sinks_helpers: LinkedList<FlowCalculatorType>,
        mut recipes_helpers: Vec<(usize, LinkedList<FlowCalculatorType>)>,
    ) -> LinkedList<FlowCalculatorType> {
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
        calculate_helper
    }

    fn reset_flows(&mut self) {
        //reset flows
        for source in self.sources.iter_mut() {
            source.output.reset();
        }

        for recipe in self.simple_recipes.iter_mut() {
            for input in recipe.inner_recipe.inputs.iter_mut() {
                match input {
                    ManageFlow::RecipeInput(r) => r.reset(),
                    ManageFlow::RecipeOutput(_) => {}
                }
            }
            for output in recipe.inner_recipe.outputs.iter_mut() {
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
    }

    ///Perform the calculation from the calculate helpers
    fn perform_calculation(&mut self, calculate_helper: &mut LinkedList<FlowCalculatorType>) {
        for calculate_helper in calculate_helper.iter_mut() {
            match calculate_helper {
                FlowCalculatorType::Helper(h) => match h.start_type {
                    RecipeWindowType::SimpleRecipe => {
                        let source = &mut self.simple_recipes[h.start_window_index];
                        let source_flow = &mut source.inner_recipe.outputs[h.start_flow_index];
                        match source_flow {
                            ManageFlow::RecipeInput(_) => {
                                println!("Resource source wrong")
                            }
                            ManageFlow::RecipeOutput(o) => {
                                let used_flow = o.created.clone();
                                let added_source = o.add_out_flow(used_flow.clone());

                                let end_flow = match h.end_type {
                                    RecipeWindowType::SimpleRecipe => {
                                        let end = &mut self.simple_recipes[h.end_window_index];
                                        let f = match &mut end.inner_recipe.inputs[h.end_flow_index]
                                        {
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
                                    RecipeWindowType::CompoundRecipe => {
                                        let end = &mut self.compound_recipes[h.end_window_index];
                                        let f = match &mut end.inner_recipe.inputs[h.end_flow_index]
                                        {
                                            ManageFlow::RecipeInput(r) => r,
                                            ManageFlow::RecipeOutput(_) => {
                                                panic!("Never happens")
                                            }
                                        };
                                        Some(f)
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
                        let source = &mut self.sources[h.start_window_index];
                        let end_flow = match h.end_type {
                            RecipeWindowType::SimpleRecipe => {
                                let end = &mut self.simple_recipes[h.end_window_index];
                                let f = match &mut end.inner_recipe.inputs[h.end_flow_index] {
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
                            RecipeWindowType::CompoundRecipe => {
                                let end = &mut self.compound_recipes[h.end_window_index];
                                let f = match &mut end.inner_recipe.inputs[h.end_flow_index] {
                                    ManageFlow::RecipeInput(r) => r,
                                    ManageFlow::RecipeOutput(_) => {
                                        panic!("Never happens")
                                    }
                                };
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
                    RecipeWindowType::CompoundRecipe => {
                        let source = &mut self.compound_recipes[h.start_window_index];
                        let source_flow = &mut source.inner_recipe.outputs[h.start_flow_index];
                        match source_flow {
                            ManageFlow::RecipeInput(_) => {
                                println!("Resource source wrong")
                            }
                            ManageFlow::RecipeOutput(o) => {
                                let used_flow = o.created.clone();
                                let added_source = o.add_out_flow(used_flow.clone());

                                let end_flow = match h.end_type {
                                    RecipeWindowType::SimpleRecipe => {
                                        let end = &mut self.simple_recipes[h.end_window_index];
                                        let f = match &mut end.inner_recipe.inputs[h.end_flow_index]
                                        {
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
                                    RecipeWindowType::CompoundRecipe => {
                                        let end = &mut self.compound_recipes[h.end_window_index];
                                        let f = match &mut end.inner_recipe.inputs[h.end_flow_index]
                                        {
                                            ManageFlow::RecipeInput(r) => r,
                                            ManageFlow::RecipeOutput(_) => {
                                                panic!("Never happens")
                                            }
                                        };
                                        Some(f)
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
    use crate::app::recipe_window::resource_sink::ResourceSink;
    use crate::app::recipe_window::resources_sources::ResourceSource;
    use crate::app::recipe_window::simple_recipe_window::tests::setup_basic_recipe_one_to_one;
    use crate::app::recipe_window::RecipeWindowType;
    use crate::app::resources::resource_flow::test::setup_flow_resource_a;
    use eframe::epaint::ahash::HashMapExt;

    use crate::app::recipe_graph::RecipeGraph;
    use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
    use crate::app::resources::{RatePer, ResourceDefinition};
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
                recipe_1t1.recipe.inner_recipe.id,
                RecipeWindowType::SimpleRecipe,
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
                    recipe_1t1.recipe.inner_recipe.id,
                    RecipeWindowType::SimpleRecipe,
                    0,
                )
                .expect("arrow error");
            graph.simple_recipes.push(recipe_1t1.recipe);
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
                println!("sink{sink:?}");
                let flow = sink
                    .sink
                    .as_ref()
                    .expect("Unconnected calculation sink")
                    .total_in();
                result.insert(flow.resource, (flow.amount, flow.rate));
            }

            result
        }
    }
    // ------------------------------- Test -------------------------------

    #[test]
    fn test_calculation() {
        let test_infos = setup_test_graphs();

        for test_info in test_infos {
            println!("-------------------------------------------------------------------------");
            println!("Start test on graph: {}", test_info.name);
            let mut graph = test_info.graph;
            graph.calculate();
            let calculated_inputs = graph.get_calc_sources();
            println!("Calculated inputs: {calculated_inputs:?}");
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
            let calculated_outputs = graph.get_calc_sinks();
            println!("Calculated outputs: {calculated_inputs:?}");
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
    }

    fn setup_test_graphs() -> [TestInfo; 2] {
        [
            RecipeGraph::setup_empty_graph(),
            RecipeGraph::setup_simple_graph(),
        ]
    }
}
