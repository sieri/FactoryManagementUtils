use crate::utils::Number;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

const TICKS_TO_SECONDS: usize = 20;
const SECONDS_TO_MINUTES: usize = 60;
const MINUTES_TO_HOURS: usize = 60;

///unit of a resource
/// * PIECES normal objects
/// * LITER volume measurement
/// * KG weight measurement
#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum Unit {
    Piece,
    Liter,
    Kg,
}
///rate of a flow
#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum RatePer {
    Tick,
    Second,
    Minute,
    Hour,
}

///A type of a resource
#[derive(PartialEq, PartialOrd, Clone, serde::Deserialize, serde::Serialize)]
pub struct ResourceDefinition {
    ///The name of the resource, should be unique
    pub name: String,

    ///the unit of that resource
    pub unit: Unit,
}

///A flow of resource
#[derive(PartialEq, PartialOrd, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ResourceFlow<T: Number> {
    pub resource: ResourceDefinition,
    pub amount: T,
    pub rate: RatePer,
}

impl<T: Number> ResourceFlow<T> {
    pub fn new(resource: &ResourceDefinition, amount: T, rate: RatePer) -> ResourceFlow<T> {
        Self {
            resource: resource.clone(),
            amount,
            rate,
        }
    }

    pub fn empty(resource: &ResourceDefinition, rate: RatePer) -> ResourceFlow<T> {
        Self::new(resource, T::zero(), rate)
    }

    /// Return the amount that flow has for a different longer term rate, return an error if the
    /// rate asked is lower than the current, to prevent int division
    ///
    /// # Arguments
    ///
    /// * `rate`: the asked rate
    ///
    /// returns: Result<T, FlowError>
    pub fn convert_amount(&self, rate: RatePer) -> Result<T, FlowError> {
        match self.rate {
            RatePer::Tick => match rate {
                RatePer::Tick => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion)),
            },
            RatePer::Second => match rate {
                RatePer::Tick => Ok(self.amount * TICKS_TO_SECONDS.into()),
                RatePer::Second => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion)),
            },
            RatePer::Minute => match rate {
                RatePer::Tick => Ok(self.amount * (TICKS_TO_SECONDS * SECONDS_TO_MINUTES).into()),
                RatePer::Second => Ok(self.amount * SECONDS_TO_MINUTES.into()),
                RatePer::Minute => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion)),
            },
            RatePer::Hour => match rate {
                RatePer::Tick => {
                    Ok(self.amount
                        * (TICKS_TO_SECONDS * SECONDS_TO_MINUTES * MINUTES_TO_HOURS).into())
                }
                RatePer::Second => Ok(self.amount * (SECONDS_TO_MINUTES * MINUTES_TO_HOURS).into()),
                RatePer::Minute => Ok(self.amount * MINUTES_TO_HOURS.into()),
                RatePer::Hour => Ok(self.amount),
            },
        }
    }

    /// Convert a flow flow has for a different longer term rate, return an error if the
    /// rate asked is lower than the current, to prevent int division
    ///
    /// # Arguments
    ///
    /// * `rate`:
    ///
    /// returns: Result<_, FlowError>
    pub fn convert(&mut self, rate: RatePer) -> Result<(), FlowError> {
        match self.convert_amount(rate) {
            Ok(amount) => {
                self.rate = rate;
                self.amount = amount;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///Add the flow to the current flow
    ///
    /// # Arguments
    ///
    /// * `other`: the other flow to add.
    ///
    /// returns: ()
    pub fn add(&mut self, other: &Self) {
        if self.resource != other.resource {
            // doesn't add if the resource doesn't match
            return;
        }

        if self.rate == other.rate {
            self.amount += other.amount;
        } else if self.rate > other.rate {
            self.amount += other.convert_amount(self.rate).unwrap();
        } else {
            let _ = self.convert(other.rate);
            self.amount += other.amount;
        }
    }
}

///generic trait for any structure that manage a resource flow
pub(crate) trait ManageResourceFlow<T: Number> {
    /// Add a amount to flow inside the resource container
    ///
    /// # Arguments
    ///
    /// * `ResourceFlow`: The flow to add as an input
    ///
    /// returns: `bool` flag indicating if the flow has been added correctly
    ///
    /// # Examples
    ///
    /// ```
    /// let added: bool = container.add_in_flow(ResourceFlow(resource, 100, PerSecond));
    /// ```
    fn add_in_flow(&mut self, flow: ResourceFlow<T>) -> bool;

    /// Add a amount to flow inside the resource container
    ///
    /// # Arguments
    ///
    /// * `ResourceFlow`: The flow to add as an output
    ///
    /// returns: `bool` flag indicating if the flow has been added correctly
    ///
    /// # Examples
    ///
    /// ```
    /// let added: bool = container.add_out_flow(ResourceFlow(resource, 100, PerSecond));
    /// ```
    fn add_out_flow(&mut self, flow: ResourceFlow<T>) -> bool;

    ///return the total flow in
    fn total_in(&self) -> ResourceFlow<T>;

    /// return the total flow out
    fn total_out(&self) -> ResourceFlow<T>;

    ///indicate the flow is enough
    fn is_enough(&self) -> bool {
        self.total_out() < self.total_in()
    }

    fn resource(&self) -> ResourceDefinition;
}

///an input resource for a recipe
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct RecipeInputResource<T: Number> {
    ///the type of resource this considers
    resource: ResourceDefinition,

    ///inputs flows
    inputs: Vec<ResourceFlow<T>>,

    ///amount needed per recipe cycle
    needed: ResourceFlow<T>,
}

///an input resource for a recipe
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct RecipeOutputResource<T: Number> {
    ///the type of resource this considers
    resource: ResourceDefinition,

    ///outputs flows
    outputs: Vec<ResourceFlow<T>>,

    ///amount created per recipe cycle
    created: ResourceFlow<T>,
}

impl<T: Number> RecipeInputResource<T> {
    pub(crate) fn new(resource: ResourceDefinition, needed: ResourceFlow<T>) -> Self {
        Self {
            resource,
            inputs: vec![],
            needed,
        }
    }
}

impl<T: Number> RecipeOutputResource<T> {
    pub(crate) fn new(resource: ResourceDefinition, created: ResourceFlow<T>) -> Self {
        Self {
            resource,
            outputs: vec![],
            created,
        }
    }
}

impl<T: Number> ManageResourceFlow<T> for RecipeInputResource<T> {
    fn add_in_flow(&mut self, flow: ResourceFlow<T>) -> bool {
        if flow.resource != self.resource {
            return false;
        }

        self.inputs.push(flow);
        true
    }

    fn add_out_flow(&mut self, _flow: ResourceFlow<T>) -> bool {
        false
    }

    fn total_in(&self) -> ResourceFlow<T> {
        let rate = self.needed.rate;
        let definition = &self.resource;
        let mut flow = ResourceFlow::empty(definition, rate);
        for input in self.inputs.iter() {
            flow.add(input);
        }

        flow
    }

    fn total_out(&self) -> ResourceFlow<T> {
        self.needed.clone()
    }

    fn resource(&self) -> ResourceDefinition {
        self.resource.clone()
    }
}

impl<T: Number> ManageResourceFlow<T> for RecipeOutputResource<T> {
    fn add_in_flow(&mut self, _flow: ResourceFlow<T>) -> bool {
        false
    }

    fn add_out_flow(&mut self, flow: ResourceFlow<T>) -> bool {
        if flow.resource != self.resource {
            return false;
        }

        self.outputs.push(flow);
        true
    }

    fn total_in(&self) -> ResourceFlow<T> {
        self.created.clone()
    }

    fn total_out(&self) -> ResourceFlow<T> {
        let rate = self.created.rate;
        let definition = &self.resource;
        let mut flow = ResourceFlow::empty(definition, rate);
        for output in self.outputs.iter() {
            flow.add(output);
        }

        flow
    }

    fn resource(&self) -> ResourceDefinition {
        self.resource.clone()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) enum ManageFlow<T: Number> {
    RecipeInput(RecipeInputResource<T>),
    RecipeOutput(RecipeOutputResource<T>),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlowErrorType {
    RateTooLowConversion,
    WrongResourceType,
}

pub(crate) struct FlowError {
    error_type: FlowErrorType,
}

impl FlowError {
    pub(crate) fn new(error_type: FlowErrorType) -> Self {
        Self { error_type }
    }

    pub(crate) fn str(&self) -> &str {
        match self.error_type {
            FlowErrorType::RateTooLowConversion => "Rate type too low for rate conversion",
            FlowErrorType::WrongResourceType => "Resource Flow of wrong type",
        }
    }
}

impl Debug for FlowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlowError ({})", self.str())
    }
}

impl Display for FlowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlowError ({})", self.str())
    }
}

impl Error for FlowError {}
