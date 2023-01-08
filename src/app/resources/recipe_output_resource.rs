use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ResourceDefinition;
use crate::utils;
use crate::utils::Number;

///an input resource for a recipe
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct RecipeOutputResource<T: Number> {
    ///the type of resource this considers
    resource: ResourceDefinition,

    ///outputs flows
    outputs: Vec<ResourceFlow<T, f32>>,

    ///amount created per recipe cycle
    pub(crate) created: ResourceFlow<T, f32>,
}

impl<T: Number> RecipeOutputResource<T> {
    pub(crate) fn new(resource: ResourceDefinition, created: ResourceFlow<T, f32>) -> Self {
        Self {
            resource,
            outputs: vec![],
            created,
        }
    }
}

impl<T: Number> ManageResourceFlow<T> for RecipeOutputResource<T> {
    fn add_in_flow(&mut self, _flow: ResourceFlow<T, f32>) -> bool {
        false
    }

    fn add_out_flow(&mut self, flow: ResourceFlow<T, f32>) -> bool {
        if flow.resource != self.resource {
            return false;
        }
        self.outputs.push(flow);
        true
    }

    fn total_in(&self) -> ResourceFlow<T, f32> {
        self.created.clone()
    }

    fn total_out(&self) -> ResourceFlow<T, f32> {
        let rate = self.created.rate;
        let definition = &self.resource;
        let mut flow = ResourceFlow::empty(definition, rate);
        for output in self.outputs.iter() {
            flow.add(output);
        }

        flow
    }

    fn is_enough(&self) -> bool {
        // println!(
        //     "Is enough?\n name {} \n total_out {} \n created {}",
        //     self.resource.name,
        //     self.total_out(),
        //     self.created
        // );
        self.total_out() <= self.created
    }

    fn resource(&self) -> ResourceDefinition {
        self.resource.clone()
    }

    fn set_designed_amount_per_cycle(&mut self, amount: T) {
        self.created.amount_per_cycle = amount;
    }

    fn to_string(&self) -> String {
        let strings = self.to_split_string();
        format!("{}: {}||{}", strings[0], strings[1], strings[2])
    }

    fn to_split_string(&self) -> [String; 3] {
        [
            self.resource.name.clone(),
            format!("{}", self.created.amount_per_cycle),
            format!(
                "{}{}",
                utils::float_format(self.created.amount, 3),
                self.created.rate.to_shortened_string()
            ),
        ]
    }

    fn reset(&mut self) {
        self.outputs.clear();
    }
}
