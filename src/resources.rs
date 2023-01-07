use crate::utils::{FloatingNumber, Number};

use crate::utils;
use num_traits::NumCast;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

const TICKS_TO_SECONDS: f32 = 20.0;
const SECONDS_TO_MINUTES: f32 = 60.0;
const MINUTES_TO_HOURS: f32 = 60.0;

///unit of a resource
/// * PIECES normal objects
/// * LITER volume measurement
/// * KG weight measurement
#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum Unit {
    Piece,
    Liter,
    Kg,
}
///rate of a flow
#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum RatePer {
    Tick,
    Second,
    Minute,
    Hour,
}

impl RatePer {
    pub fn next(self) -> Self {
        match self {
            RatePer::Tick => RatePer::Second,
            RatePer::Second => RatePer::Minute,
            RatePer::Minute => RatePer::Hour,
            RatePer::Hour => {
                panic!("Can't call next on hour")
            }
        }
    }

    pub fn to_shortened_string(&self) -> String {
        match self {
            RatePer::Tick => "/tick",
            RatePer::Second => "/s",
            RatePer::Minute => "/min ",
            RatePer::Hour => "/h",
        }
        .to_string()
    }
}

impl Display for RatePer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RatePer::Tick => {
                write!(f, "Per Tick")
            }
            RatePer::Second => {
                write!(f, "Per Second")
            }
            RatePer::Minute => {
                write!(f, "Per Minute")
            }
            RatePer::Hour => {
                write!(f, "Per Hour")
            }
        }
    }
}

///A type of a resource
#[derive(Debug, PartialEq, PartialOrd, Clone, serde::Deserialize, serde::Serialize)]
pub struct ResourceDefinition {
    ///The name of the resource, should be unique
    pub name: String,

    ///the unit of that resource
    pub unit: Unit,
}

impl Display for ResourceDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource(name={})", self.name)?;
        Ok(())
    }
}

///A flow of resource
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ResourceFlow<T: Number, F: FloatingNumber> {
    pub resource: ResourceDefinition,
    pub amount_per_cycle: T,
    pub amount: F,
    pub rate: RatePer,
}

impl<T: Number, F: FloatingNumber> PartialOrd for ResourceFlow<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.resource != other.resource {
            return None;
        }

        if self.rate > other.rate {
            let amount = other.convert_amount(self.rate).unwrap();
            self.amount.partial_cmp(&amount)
        } else {
            self.convert_amount(other.rate)
                .unwrap()
                .partial_cmp(&other.amount)
        }
    }
}

impl<T: Number, F: FloatingNumber> Display for ResourceFlow<T, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ResourceFlow(resource={}, amount_per_cycle={}, amount={}, rate={})",
            self.resource, self.amount_per_cycle, self.amount, self.rate
        )?;
        Ok(())
    }
}

impl<T: Number, F: FloatingNumber> ResourceFlow<T, F> {
    pub fn new(
        resource: &ResourceDefinition,
        amount_per_cycle: T,
        amount: F,
        rate: RatePer,
    ) -> ResourceFlow<T, F> {
        Self {
            resource: resource.clone(),
            amount_per_cycle,
            amount,
            rate,
        }
    }

    pub fn empty(resource: &ResourceDefinition, rate: RatePer) -> ResourceFlow<T, F> {
        Self::new(resource, T::zero(), F::zero(), rate)
    }

    /// Return the amount that flow has for a different longer term rate, return an error if the
    /// rate asked is lower than the current, to prevent int division
    ///
    /// # Arguments
    ///
    /// * `rate`: the asked rate
    ///
    /// returns: Result<T, FlowError>
    pub fn convert_amount(&self, rate: RatePer) -> Result<F, FlowError> {
        match rate {
            RatePer::Tick => match self.rate {
                RatePer::Tick => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion(
                    self.rate, rate,
                ))),
            },
            RatePer::Second => match self.rate {
                RatePer::Tick => Ok(self.amount * TICKS_TO_SECONDS.into()),
                RatePer::Second => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion(
                    self.rate, rate,
                ))),
            },
            RatePer::Minute => match self.rate {
                RatePer::Tick => Ok(self.amount * (TICKS_TO_SECONDS * SECONDS_TO_MINUTES).into()),
                RatePer::Second => Ok(self.amount * SECONDS_TO_MINUTES.into()),
                RatePer::Minute => Ok(self.amount),
                _ => Err(FlowError::new(FlowErrorType::RateTooLowConversion(
                    self.rate, rate,
                ))),
            },
            RatePer::Hour => match self.rate {
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

    ///Convert new time base
    pub fn convert_time_base(&mut self, cycle_length: T, time: RatePer) -> Result<(), FlowError> {
        let amount_per_cycle: F = NumCast::from(self.amount_per_cycle).unwrap();
        let new_amount: F = amount_per_cycle / NumCast::from(cycle_length).unwrap();
        self.rate = time;
        self.amount = new_amount;

        while self.amount < F::one() && self.rate < RatePer::Hour {
            self.convert(self.rate.next())?;
        }
        Ok(())
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
    fn add_in_flow(&mut self, flow: ResourceFlow<T, f32>) -> bool;

    /// Add a amount to flow inside the resource container
    ///
    /// # Arguments
    ///
    /// * `ResourceFlow`: The flow to add as an output
    ///
    /// returns: `bool` flag indicating if the flow has been added correctly
    fn add_out_flow(&mut self, flow: ResourceFlow<T, f32>) -> bool;

    ///return the total flow in
    fn total_in(&self) -> ResourceFlow<T, f32>;

    /// return the total flow out
    fn total_out(&self) -> ResourceFlow<T, f32>;

    ///indicate the flow is enough
    fn is_enough(&self) -> bool;

    ///the ``ResourceDefinition`` representing the flow
    fn resource(&self) -> ResourceDefinition;

    fn set_designed_amount_per_cycle(&mut self, amount: T);

    ///Give a string representation
    fn to_string(&self) -> String;

    ///Give a string representation split in three strings for custom formatting first the name,
    ///then the amount per cycle then the amount of per time, with unit
    fn to_split_string(&self) -> [String; 3];

    ///reset the flows
    fn reset(&mut self);
}

///an input resource for a recipe
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct RecipeInputResource<T: Number> {
    ///the type of resource this considers
    resource: ResourceDefinition,

    ///inputs flows
    inputs: Vec<ResourceFlow<T, f32>>,

    ///amount needed per recipe cycle
    pub(crate) needed: ResourceFlow<T, f32>,
}

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

impl<T: Number> RecipeInputResource<T> {
    pub(crate) fn new(resource: ResourceDefinition, needed: ResourceFlow<T, f32>) -> Self {
        Self {
            resource,
            inputs: vec![],
            needed,
        }
    }
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

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) enum ManageFlow<T: Number> {
    RecipeInput(RecipeInputResource<T>),
    RecipeOutput(RecipeOutputResource<T>),
}

impl<T: Number> ManageFlow<T> {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            ManageFlow::RecipeInput(input) => input.to_string(),
            ManageFlow::RecipeOutput(output) => output.to_string(),
        }
    }

    pub fn to_split_string(&self) -> [String; 3] {
        match self {
            ManageFlow::RecipeInput(input) => input.to_split_string(),
            ManageFlow::RecipeOutput(output) => output.to_split_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlowErrorType {
    RateTooLowConversion(RatePer, RatePer),
    WrongResourceType,
}

pub(crate) struct FlowError {
    error_type: FlowErrorType,
}

impl FlowError {
    pub(crate) fn new(error_type: FlowErrorType) -> Self {
        Self { error_type }
    }

    pub(crate) fn str(&self) -> String {
        match self.error_type {
            FlowErrorType::RateTooLowConversion(start, end) => {
                format!("Rate type too low for rate conversion: from {start} to {end}")
            }
            FlowErrorType::WrongResourceType => "Resource Flow of wrong type".to_string(),
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
