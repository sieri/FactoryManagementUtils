use crate::utils::Number;

use recipe_input_resource::RecipeInputResource;
use recipe_output_resource::RecipeOutputResource;
use resource_flow::ManageResourceFlow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod recipe_input_resource;
pub mod recipe_output_resource;
pub mod resource_flow;

const TICKS_TO_SECONDS: f32 = 20.0;
const SECONDS_TO_MINUTES: f32 = 60.0;
const MINUTES_TO_HOURS: f32 = 60.0;

///unit of a resource
/// * PIECES normal objects
/// * LITER volume measurement
/// * KG weight measurement
#[allow(dead_code)]
#[derive(
    Debug, PartialEq, Eq, Hash, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum Unit {
    Piece,
    Liter,
    Kg,
}
///rate of a flow
#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, serde::Deserialize, serde::Serialize, Eq)]
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

    pub fn to_shortened_string(self) -> String {
        match self {
            RatePer::Tick => "/tick",
            RatePer::Second => "/s",
            RatePer::Minute => "/min",
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
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Clone, serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub(crate) enum ManageFlow<T: Number> {
    RecipeInput(RecipeInputResource<T>),
    RecipeOutput(RecipeOutputResource<T>),
}

impl<T: Number> ManageFlow<T> {
    #[allow(dead_code)]
    pub fn to_string_rep(&self) -> String {
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

#[cfg(test)]
pub mod test {

    use crate::app::resources::{ResourceDefinition, Unit};

    pub(crate) fn setup_resource_a() -> ResourceDefinition {
        ResourceDefinition {
            name: "Resource A".to_string(),
            unit: Unit::Piece,
        }
    }

    //-------------------Tests-------------------
}
