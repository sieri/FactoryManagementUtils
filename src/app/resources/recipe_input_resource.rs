use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ResourceDefinition;
use crate::utils;
use crate::utils::Number;

///an input resource for a recipe
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug,  PartialEq)]
pub(crate) struct RecipeInputResource<T: Number> {
    ///the type of resource this considers
    resource: ResourceDefinition,

    ///inputs flows
    inputs: Vec<ResourceFlow<T, f32>>,

    ///amount needed per recipe cycle
    pub(crate) needed: ResourceFlow<T, f32>,
}

impl<T:Number> Eq for RecipeInputResource<T> {
    
}

impl<T: Number> RecipeInputResource<T> {
    pub(crate) fn new(resource: ResourceDefinition, needed: ResourceFlow<T, f32>) -> Self {
        Self {
            resource,
            inputs: vec![],
            needed,
        }
    }
}

impl<T: Number> ManageResourceFlow<T> for RecipeInputResource<T> {
    fn add_in_flow(&mut self, flow: ResourceFlow<T, f32>) -> bool {
        if flow.resource != self.resource {
            return false;
        }
        self.inputs.push(flow);
        true
    }

    fn add_out_flow(&mut self, _flow: ResourceFlow<T, f32>) -> bool {
        false
    }

    fn total_in(&self) -> ResourceFlow<T, f32> {
        let rate = self.needed.rate;
        let definition = &self.resource;
        let mut flow = ResourceFlow::empty(definition, rate);
        for input in self.inputs.iter() {
            flow.add(input);
        }

        flow
    }

    fn total_out(&self) -> ResourceFlow<T, f32> {
        self.needed.clone()
    }

    fn is_enough(&self) -> bool {
        self.total_in() >= self.needed
    }

    fn resource(&self) -> ResourceDefinition {
        self.resource.clone()
    }

    fn set_designed_amount_per_cycle(&mut self, amount: T) {
        self.needed.amount_per_cycle = amount;
    }

    fn to_string(&self) -> String {
        let strings = self.to_split_string();
        format!("{}: {}||{}", strings[0], strings[1], strings[2])
    }

    fn to_split_string(&self) -> [String; 3] {
        [
            self.resource.name.clone(),
            format!("{}", self.needed.amount_per_cycle),
            format!(
                "{}{}",
                utils::float_format(self.needed.amount, 3),
                self.needed.rate.to_shortened_string()
            ),
        ]
    }

    fn reset(&mut self) {
        self.inputs.clear()
    }
}
