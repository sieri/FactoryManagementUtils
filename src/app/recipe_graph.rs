use crate::app::recipe_window::arrow_flow::ArrowFlow;
use crate::app::recipe_window::base_recipe_window::RecipeWindowUser;
use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;
use crate::app::recipe_window::resource_sink::ResourceSink;
use crate::app::recipe_window::resources_sources::ResourceSource;
use crate::app::recipe_window::simple_recipe_window::SimpleRecipeWindow;
use crate::app::recipe_window::RecipeWindowType;
use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ManageFlow;
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        info!("==================Calculate==================");

        self.reset_flows();

        let mut calculate_helper = self.make_helpers();

        self.perform_calculation(&mut calculate_helper);

        self.back_propagation(&mut calculate_helper);
    }

    fn make_helpers(&mut self) -> LinkedList<FlowCalculatorType> {
        let mut sources_helpers = LinkedList::new();
        let mut sinks_helpers = LinkedList::new();
        let mut simple_recipes_helpers = vec![(0, LinkedList::new()); self.simple_recipes.len()];
        let mut compound_recipes_helpers =
            vec![(0, LinkedList::new()); self.compound_recipes.len()];

        trace!("build relationships from arrows");
        for arrow in self.arrows.clone().iter() {
            let (start_flow_index, start_type, start_window_index) = self.get_start_point(arrow);

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
                            error!("Starting an arrow flow at a sink, this isn't normal")
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

        Self::concatenate_helpers(
            sources_helpers,
            sinks_helpers,
            simple_recipes_helpers,
            compound_recipes_helpers,
        )
    }

    fn connect_resources_helpers_ends(
        simple_recipes_helpers: &mut [(usize, LinkedList<FlowCalculatorType>)],
        compound_recipes_helpers: &mut [(usize, LinkedList<FlowCalculatorType>)],
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
                error!("Ending an arrow flow at a source, this doesn't shouldn't happen")
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

    fn get_start_point(&mut self, arrow: &ArrowFlow) -> (usize, RecipeWindowType, Option<usize>) {
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
                error!("Incorrect start point type, can't be a sink");
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
        mut compound_recipes_helpers: Vec<(usize, LinkedList<FlowCalculatorType>)>,
    ) -> LinkedList<FlowCalculatorType> {
        trace!("Concatenate helpers");

        let mut calculate_helper = sources_helpers;

        for (i, (_, list)) in recipes_helpers.iter_mut().enumerate() {
            list.push_back(FlowCalculatorType::EndRecipe(
                i,
                RecipeWindowType::SimpleRecipe,
            ));
        }
        for (i, (_, list)) in compound_recipes_helpers.iter_mut().enumerate() {
            list.push_back(FlowCalculatorType::EndRecipe(
                i,
                RecipeWindowType::CompoundRecipe,
            ));
        }
        recipes_helpers.append(&mut compound_recipes_helpers);
        recipes_helpers.sort_by(|helper1, helper2| helper1.0.partial_cmp(&helper2.0).unwrap());
        println!("Hey! {recipes_helpers:?}");
        for (_i, (_, list)) in recipes_helpers.iter_mut().enumerate() {
            calculate_helper.append(list);
        }
        calculate_helper.append(&mut sinks_helpers);
        calculate_helper
    }

    fn reset_flows(&mut self) {
        trace!("Reset flows");
        //reset flows
        for source in self.sources.iter_mut() {
            trace!("reset source {:?}", source.output);
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
        trace!("Done resetting!")
    }

    ///Perform the calculation from the calculate helpers
    fn perform_calculation(&mut self, calculate_helper: &mut LinkedList<FlowCalculatorType>) {
        trace!("Perform Calculation");
        for calculate_helper in calculate_helper.iter_mut() {
            match calculate_helper {
                FlowCalculatorType::Helper(h) => match h.start_type {
                    RecipeWindowType::SimpleRecipe => {
                        let source = &mut self.simple_recipes[h.start_window_index];
                        let source_flow = &mut source.inner_recipe.outputs[h.start_flow_index];
                        match source_flow {
                            ManageFlow::RecipeInput(_) => {
                                error!("Source flows shouldn't be a RecipeInput")
                            }
                            ManageFlow::RecipeOutput(o) => {
                                //source is simple recipe
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
                                    error!("added_source:{added_source} added_inputs{added_input}");
                                }
                            }
                        }
                    }
                    RecipeWindowType::Source => {
                        //source is a source
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

                        //connect the end to the source when the source is a source
                        if let Some(end_flow) = end_flow {
                            if source.limited_output {
                                let resource = source.output.created.resource.clone();
                                let used_flow = ResourceFlow::new(
                                    &resource,
                                    1,
                                    source.limit_amount,
                                    source.limit_rate,
                                );
                                add_flows(&mut source.output, end_flow, used_flow);
                            } else {
                                let used_flow = end_flow.needed.clone();
                                debug!("Flow used for output of a source {}", used_flow);
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
                                error!("Source flow shouldn't be a RecipeInput")
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
                                    error!("added_source:{added_source} added_inputs{added_input}");
                                }
                            }
                        }
                    }
                },
                FlowCalculatorType::EndRecipe(i, recipe_type) => match recipe_type {
                    RecipeWindowType::SimpleRecipe => {
                        self.simple_recipes[*i].internal_calculation();
                    }
                    RecipeWindowType::CompoundRecipe => {
                        self.compound_recipes[*i].internal_calculation();
                    }
                    RecipeWindowType::Source => {}
                    RecipeWindowType::Sink => {}
                },
            }
        }
    }

    /// Get the rate to limit a flow in back propagation according to the end point of a helper
    ///
    /// # Arguments
    ///
    /// * `helper`: the helper to read
    ///
    /// returns: rate of limitation, optional amount information
    fn get_back_rate(
        &mut self,
        helper: FlowCalculatorType,
    ) -> (f32, Option<ResourceFlow<usize, f32>>) {
        match helper {
            FlowCalculatorType::Helper(h) => match h.end_type {
                RecipeWindowType::SimpleRecipe => {
                    let end = &mut self.simple_recipes[h.end_window_index];
                    let end_flow = &mut end.inner_recipe.inputs[h.end_flow_index];

                    match end_flow {
                        ManageFlow::RecipeInput(i) => {
                            debug!(
                                "get back rate Resource: {}\n inputs! {}{}\n outputs! {}{}",
                                i.resource().name,
                                i.total_in().amount,
                                i.total_in().rate.to_shortened_string(),
                                i.total_out().amount,
                                i.total_out().rate.to_shortened_string()
                            );
                            let rate_res = i.total_out() / i.total_in();
                            match rate_res {
                                Ok(rate) => {
                                    return (rate, Some(i.total_out()));
                                }
                                Err(error) => {
                                    error!("Flow back propagation can't be calculated due to a '{}' on rate calculation", error.str())
                                }
                            }
                        }
                        ManageFlow::RecipeOutput(_) => {
                            error!("End flow shouldn't be a RecipeOutput")
                        }
                    }
                }
                RecipeWindowType::CompoundRecipe => {
                    let end = &mut self.compound_recipes[h.end_window_index];
                    let end_flow = &mut end.inner_recipe.inputs[h.end_window_index];

                    match end_flow {
                        ManageFlow::RecipeInput(i) => {
                            if i.is_more_than_enough() {
                                let rate = i.total_out().amount / i.total_in().amount;
                                return (rate, Some(i.total_out() * rate));
                            }
                        }
                        ManageFlow::RecipeOutput(_) => {
                            error!("End flow shouldn't be a RecipeOutput")
                        }
                    }
                }
                RecipeWindowType::Source => {
                    error!("End of a flow shouldn't be a Source")
                }
                RecipeWindowType::Sink => {}
            },
            FlowCalculatorType::EndRecipe(_, _) => {}
        };
        (1.0, None)
    }

    fn back_propagation(&mut self, calculate_helper: &mut LinkedList<FlowCalculatorType>) {
        debug!("Back propagation");
        //reverse
        let mut reverse_helper = LinkedList::new();

        for _ in 0..calculate_helper.len() {
            let calc = calculate_helper.pop_back();
            if let Some(calc) = calc {
                reverse_helper.push_back(calc);
            }
        }

        for helper in reverse_helper {
            let (rate, flow) = self.get_back_rate(helper);
            debug!(
                "Rate of back propagation {}, associated flow={:?}",
                rate, flow
            );
            if rate == 1.0 {
                continue;
            }
            match helper {
                FlowCalculatorType::Helper(h) => match h.start_type {
                    RecipeWindowType::SimpleRecipe => {
                        let start = &mut self.simple_recipes[h.start_window_index];
                        start.back_propagation_internal_calculation(rate, flow)
                    }
                    RecipeWindowType::CompoundRecipe => {
                        let start = &mut self.compound_recipes[h.start_window_index];
                        start.back_propagation_internal_calculation(rate, flow)
                    }
                    RecipeWindowType::Source => {
                        let start = &mut self.sources[h.start_window_index];
                        if let Some(flow) = flow {
                            if flow < start.output.total_out() {
                                debug!("source flow back propagated {}", flow);
                                start.output.reset();
                                start.output.add_out_flow(flow);
                            }
                        } else {
                            panic!("Hey! you need a flow to set it")
                        }
                    }
                    RecipeWindowType::Sink => {}
                },
                FlowCalculatorType::EndRecipe(_, _) => {}
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
        error!("added_source:{added_source} added_inputs{added_input}");
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub mod tests {
    use crate::app::recipe_graph::RecipeGraph;
    use crate::app::recipe_window::arrow_flow::ArrowFlow;
    use crate::app::recipe_window::compound_recipe_window::CompoundRecipeWindow;
    use crate::app::recipe_window::resource_sink::ResourceSink;
    use crate::app::recipe_window::resources_sources::ResourceSource;
    use crate::app::recipe_window::simple_recipe_window::tests::setup_simple_recipe_one_to_one_custom;
    use crate::app::recipe_window::test::{setup_resource_input, setup_resource_output};
    use crate::app::recipe_window::RecipeWindowType;
    use crate::app::resources::resource_flow::test::{setup_flow_resource, setup_flow_resource_a};
    use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
    use crate::app::resources::test::setup_resource;

    use crate::app::resources::{RatePer, ResourceDefinition};
    use crate::utils::test_env;
    use eframe::epaint::ahash::HashMapExt;
    use egui::epaint::ahash::HashMap;
    use egui::{LayerId, Order};
    use log::{debug, info};
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub(crate) struct TestInfo {
        pub name: String,
        pub graph: RecipeGraph,
        pub inputs: Vec<ResourceFlow<usize, f32>>,
        pub outputs: Vec<ResourceFlow<usize, f32>>,
    }

    impl Display for TestInfo {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "\nName:{}", self.name)?;
            writeln!(f, "Inputs:")?;
            for input in self.inputs.iter() {
                writeln!(f, "\t{input}")?;
            }
            writeln!(f, "Outputs:")?;
            for output in self.outputs.iter() {
                writeln!(f, "\t{output}")?;
            }
            Ok(())
        }
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
            let recipe_1t1 = setup_simple_recipe_one_to_one_custom(None, None);
            let sink = ResourceSink::new();
            let resource_a_flow = setup_flow_resource_a(None);
            let source = ResourceSource::new(resource_a_flow.resource.name.clone());
            let mut first_arrow = ArrowFlow::new(
                recipe_1t1.output_resources.first().unwrap().def.clone(),
                recipe_1t1.recipe.inner_recipe.id,
                RecipeWindowType::SimpleRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(recipe_1t1.output_resources.first().unwrap().def.clone()),
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
            let input = recipe_1t1.input_resources.first().unwrap();
            let output = recipe_1t1.output_resources.first().unwrap();
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

        pub(crate) fn setup_simple_compound_graph() -> TestInfo {
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };

            let mut graph = RecipeGraph::new();
            let recipe_compound = CompoundRecipeWindow::setup_one_to_one_compound();
            let sink = ResourceSink::new();
            let resource_a_flow = setup_flow_resource_a(None);
            let source = ResourceSource::new(resource_a_flow.resource.name.clone());
            let mut first_arrow = ArrowFlow::new(
                recipe_compound
                    .output_resources
                    .first()
                    .unwrap()
                    .def
                    .clone(),
                recipe_compound.recipe.inner_recipe.id,
                RecipeWindowType::CompoundRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(
                        recipe_compound
                            .output_resources
                            .first()
                            .unwrap()
                            .def
                            .clone(),
                    ),
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
                    recipe_compound.recipe.inner_recipe.id,
                    RecipeWindowType::CompoundRecipe,
                    0,
                )
                .expect("arrow error");
            graph.compound_recipes.push(recipe_compound.recipe);
            graph.arrows.push(first_arrow);
            graph.arrows.push(second_arrow);
            graph.sources.push(source);
            graph.sinks.push(sink);
            let input = recipe_compound.input_resources.first().unwrap();
            let output = recipe_compound.output_resources.first().unwrap();
            TestInfo {
                name: "compound one to one".to_string(),
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

        pub(crate) fn setup_simple_compound_graph_two_levels() -> TestInfo {
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };

            let mut graph = RecipeGraph::new();
            let recipe_compound = CompoundRecipeWindow::setup_one_to_one_compound_two_levels();
            let sink = ResourceSink::new();
            let resource_a_flow = setup_flow_resource_a(None);
            let source = ResourceSource::new(resource_a_flow.resource.name.clone());
            let mut first_arrow = ArrowFlow::new(
                recipe_compound
                    .output_resources
                    .first()
                    .unwrap()
                    .def
                    .clone(),
                recipe_compound.recipe.inner_recipe.id,
                RecipeWindowType::CompoundRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(
                        recipe_compound
                            .output_resources
                            .first()
                            .unwrap()
                            .def
                            .clone(),
                    ),
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
                    recipe_compound.recipe.inner_recipe.id,
                    RecipeWindowType::CompoundRecipe,
                    0,
                )
                .expect("arrow error");
            graph.compound_recipes.push(recipe_compound.recipe);
            graph.arrows.push(first_arrow);
            graph.arrows.push(second_arrow);
            graph.sources.push(source);
            graph.sinks.push(sink);
            let input = recipe_compound.input_resources.first().unwrap();
            let output = recipe_compound.output_resources.first().unwrap();
            TestInfo {
                name: "compound one to one two levels".to_string(),
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

        pub(crate) fn setup_rate_limited_graph(rate: f32) -> TestInfo {
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };
            let mut graph = RecipeGraph::new();
            let recipe_1t1 = setup_simple_recipe_one_to_one_custom(
                Some(setup_resource_input(setup_flow_resource(
                    setup_resource("Input Resource"),
                    2,
                    RatePer::Minute,
                ))),
                Some(setup_resource_output(setup_flow_resource(
                    setup_resource("Output Resource"),
                    3,
                    RatePer::Minute,
                ))),
            );
            let sink = ResourceSink::new();

            let input_resource = recipe_1t1
                .input_resources
                .first()
                .expect("Input resource error")
                .clone();
            //source
            let source = ResourceSource::limited_source(
                input_resource.def.name.clone(),
                input_resource.amount / rate,
                input_resource.rate,
            );

            let mut first_arrow = ArrowFlow::new(
                recipe_1t1.output_resources.first().unwrap().def.clone(),
                recipe_1t1.recipe.inner_recipe.id,
                RecipeWindowType::SimpleRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(recipe_1t1.output_resources.first().unwrap().def.clone()),
                    sink.id,
                    RecipeWindowType::Sink,
                    0,
                )
                .expect("arrow error");
            let mut second_arrow = ArrowFlow::new(
                input_resource.def.clone(),
                source.id,
                RecipeWindowType::Source,
                dummy_layer,
                0,
            );
            second_arrow
                .put_end(
                    Some(input_resource.def.clone()),
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
            let input = recipe_1t1.input_resources.first().unwrap();
            let output = recipe_1t1.output_resources.first().unwrap();
            TestInfo {
                name: "limited rate".to_string(),
                graph,
                inputs: vec![ResourceFlow::new(
                    &input.def.clone(),
                    input.amount_per_cycle,
                    input.amount / rate,
                    input.rate,
                )],
                outputs: vec![ResourceFlow::new(
                    &output.def.clone(),
                    output.amount_per_cycle,
                    output.amount / rate,
                    output.rate,
                )],
            }
        }

        pub(crate) fn setup_rate_limited_compound_graph() -> TestInfo {
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };

            let mut graph = RecipeGraph::new();
            let recipe_limited = CompoundRecipeWindow::setup_rate_limited_compound();
            let sink = ResourceSink::new();

            let input_resource = recipe_limited
                .input_resources
                .first()
                .expect("Input resource error")
                .clone();
            //source
            let source = ResourceSource::limited_source(
                input_resource.def.name.clone(),
                input_resource.amount / 2.0,
                input_resource.rate,
            );
            debug!("Resource source limiting: {}", input_resource.amount / 2.0);
            let mut first_arrow = ArrowFlow::new(
                recipe_limited.output_resources.first().unwrap().def.clone(),
                recipe_limited.recipe.inner_recipe.id,
                RecipeWindowType::CompoundRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(recipe_limited.output_resources.first().unwrap().def.clone()),
                    sink.id,
                    RecipeWindowType::Sink,
                    0,
                )
                .expect("arrow error");
            let mut second_arrow = ArrowFlow::new(
                input_resource.def.clone(),
                source.id,
                RecipeWindowType::Source,
                dummy_layer,
                0,
            );
            second_arrow
                .put_end(
                    Some(input_resource.def.clone()),
                    recipe_limited.recipe.inner_recipe.id,
                    RecipeWindowType::CompoundRecipe,
                    0,
                )
                .expect("arrow error");

            //add elements to the graph
            graph.compound_recipes.push(recipe_limited.recipe);
            graph.arrows.push(first_arrow);
            graph.arrows.push(second_arrow);
            graph.sources.push(source);
            graph.sinks.push(sink);

            //build the test info
            let input = recipe_limited.input_resources.first().unwrap();
            let output = recipe_limited.output_resources.first().unwrap();
            TestInfo {
                name: "compound limited rate".to_string(),
                graph,
                inputs: vec![ResourceFlow::new(
                    &input.def.clone(),
                    input.amount_per_cycle,
                    input.amount / 2.0,
                    input.rate,
                )],
                outputs: vec![ResourceFlow::new(
                    &output.def.clone(),
                    output.amount_per_cycle,
                    output.amount / 2.0,
                    output.rate,
                )],
            }
        }

        pub(crate) fn setup_back_propagation_graph() -> TestInfo {
            info!("Test setup back propagation graph🔜");
            let dummy_layer: LayerId = LayerId {
                order: Order::Background,
                id: egui::Id::new("dummy"),
            };
            let name_in_resource = "Input Resource";
            let name_mid_resource = "Middle Resource";
            let name_out_resource = "Output Resource";
            let mut graph = RecipeGraph::new();

            let first_recipe = setup_simple_recipe_one_to_one_custom(
                Some(setup_resource_input(setup_flow_resource(
                    setup_resource(name_in_resource),
                    8,
                    RatePer::Minute,
                ))),
                Some(setup_resource_output(setup_flow_resource(
                    setup_resource(name_mid_resource),
                    4,
                    RatePer::Minute,
                ))),
            );
            let second_recipe = setup_simple_recipe_one_to_one_custom(
                Some(setup_resource_input(setup_flow_resource(
                    setup_resource(name_mid_resource),
                    2,
                    RatePer::Minute,
                ))),
                Some(setup_resource_output(setup_flow_resource(
                    setup_resource(name_out_resource),
                    1,
                    RatePer::Minute,
                ))),
            );
            let sink = ResourceSink::new();

            let input_resource = first_recipe
                .input_resources
                .first()
                .expect("Input resource error")
                .clone();

            let mid_resource = second_recipe
                .input_resources
                .first()
                .expect("Input resource error")
                .clone();

            let mid_resource_output = first_recipe
                .output_resources
                .first()
                .expect("Output resource error")
                .clone();

            info!("Input resource:{:?}", input_resource);
            info!("mid resource:{:?}", mid_resource);
            info!("mid resource output:{:?}", mid_resource_output);

            assert_eq!(
                mid_resource.def, mid_resource_output.def,
                "Setup not possible, the mid resources don't match"
            );

            assert_eq!(mid_resource.rate,mid_resource_output.rate,
                       "Setup not possible, Resource output in the second recipe need to be the same resource rate for ease of first calculation");
            assert!(mid_resource.amount<mid_resource_output.amount,
                    "Setup not possible, Resource output in the second recipe need to be less than the resource input");

            let limit_rate = mid_resource.amount / mid_resource_output.amount;
            info!("Limit rate calculated:{}", limit_rate);
            //source
            let source = ResourceSource::new(input_resource.def.name.clone());

            //first arrow: second_recipe->Sink
            let mut first_arrow = ArrowFlow::new(
                second_recipe.output_resources.first().unwrap().def.clone(),
                second_recipe.recipe.inner_recipe.id,
                RecipeWindowType::SimpleRecipe,
                dummy_layer,
                0,
            );
            first_arrow
                .put_end(
                    Some(second_recipe.output_resources.first().unwrap().def.clone()),
                    sink.id,
                    RecipeWindowType::Sink,
                    0,
                )
                .expect("arrow error");

            // second arrow: first_recipe -> second_recipe
            let mut second_arrow = ArrowFlow::new(
                mid_resource.def.clone(),
                first_recipe.recipe.inner_recipe.id,
                RecipeWindowType::SimpleRecipe,
                dummy_layer,
                0,
            );
            second_arrow
                .put_end(
                    Some(mid_resource.def.clone()),
                    second_recipe.recipe.inner_recipe.id,
                    RecipeWindowType::SimpleRecipe,
                    0,
                )
                .expect("arrow error");

            // third arrow: source -> recipe1t1b
            let mut third_arrow = ArrowFlow::new(
                input_resource.def.clone(),
                source.id,
                RecipeWindowType::Source,
                dummy_layer,
                0,
            );
            third_arrow
                .put_end(
                    Some(input_resource.def.clone()),
                    first_recipe.recipe.inner_recipe.id,
                    RecipeWindowType::SimpleRecipe,
                    0,
                )
                .expect("arrow error");
            graph.simple_recipes.push(second_recipe.recipe);
            graph.simple_recipes.push(first_recipe.recipe);
            graph.arrows.push(first_arrow);
            graph.arrows.push(second_arrow);
            graph.arrows.push(third_arrow);
            graph.sources.push(source);
            graph.sinks.push(sink);

            //build info
            let input = first_recipe.input_resources.first().unwrap();
            let output = second_recipe.output_resources.first().unwrap();
            TestInfo {
                name: "back propagation".to_string(),
                graph,
                inputs: vec![ResourceFlow::new(
                    &input.def.clone(),
                    input.amount_per_cycle,
                    input.amount * limit_rate,
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
        test_env::setup();
        let tests_info = setup_test_graphs();

        for test_info in tests_info {
            info!("📍Start test on graph: {}📍", test_info.name);

            info!(
                ">\n======================{}======================",
                test_info
            );

            let mut graph = test_info.graph;
            info!("Calculate for test");
            graph.calculate();
            let calculated_inputs = graph.get_calc_sources();

            for input in test_info.inputs.iter() {
                let resource = input.resource.clone();

                let calculated = calculated_inputs
                    .get(&resource)
                    .expect("no data in the resource");

                info!(
                    "Input calculated={}{} is equal to {}?",
                    calculated.0,
                    calculated.1.to_shortened_string(),
                    input
                );
                assert_eq!(calculated.1, input.rate, "Rate of an input doesn't match");
                assert_eq!(
                    calculated.0, input.amount,
                    "Amount of an input doesn't match",
                );
            }
            let calculated_outputs = graph.get_calc_sinks();

            for output in test_info.outputs.iter() {
                let resource = output.resource.clone();
                let calculated = calculated_outputs
                    .get(&resource)
                    .expect("no data in the resource");
                info!(
                    "Output calculated={}{} is equal to {}?",
                    calculated.0,
                    calculated.1.to_shortened_string(),
                    output
                );
                assert_eq!(
                    calculated.0, output.amount,
                    "Amount of an output doesn't match",
                );
                assert_eq!(calculated.1, output.rate, "Rate of an output doesn't match");
            }
        }
    }

    pub(crate) fn setup_test_graphs() -> [TestInfo; 9] {
        [
            RecipeGraph::setup_empty_graph(),
            RecipeGraph::setup_simple_graph(),
            RecipeGraph::setup_simple_compound_graph(),
            RecipeGraph::setup_simple_compound_graph_two_levels(),
            RecipeGraph::setup_rate_limited_graph(1.0),
            RecipeGraph::setup_rate_limited_graph(2.0),
            RecipeGraph::setup_rate_limited_graph(5.0),
            RecipeGraph::setup_rate_limited_compound_graph(),
            RecipeGraph::setup_back_propagation_graph(),
        ]
    }
}

#[derive(Copy, Clone, Debug)]
enum FlowCalculatorType {
    Helper(FlowCalculatorHelper),
    EndRecipe(usize, RecipeWindowType),
}

#[derive(Copy, Clone, Debug)]
struct FlowCalculatorHelper {
    start_window_index: usize,
    start_flow_index: usize,

    end_window_index: usize,
    end_flow_index: usize,

    start_type: RecipeWindowType,
    end_type: RecipeWindowType,
}
