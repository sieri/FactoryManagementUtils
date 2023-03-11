use crate::app::resources::{
    FlowError, FlowErrorType, RatePer, ResourceDefinition, MINUTES_TO_HOURS, SECONDS_TO_MINUTES,
    TICKS_TO_SECONDS,
};
use crate::utils::{FloatingNumber, Number};

use num_traits::NumCast;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Div, Mul};

///A flow of resource
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct ResourceFlow<T: Number, F: FloatingNumber> {
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
            "\t{}: {}{}",
            self.resource,
            self.amount,
            self.rate.to_shortened_string()
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

impl<T, F> Mul<F> for ResourceFlow<T, F>
where
    T: Number,
    F: FloatingNumber,
{
    type Output = ResourceFlow<T, F>;

    fn mul(self, rhs: F) -> Self::Output {
        let mut flow = self;
        flow.amount = flow.amount * rhs;
        flow
    }
}

impl<T, F> Div for ResourceFlow<T, F>
where
    T: Number,
    F: FloatingNumber,
{
    type Output = Result<F, FlowError>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.resource != rhs.resource {
            // doesn't add if the resource doesn't match
            return Err(FlowError::new(FlowErrorType::WrongResourceType));
        }

        if self.rate == rhs.rate {
            Ok(self.amount / rhs.amount)
        } else if self.rate > rhs.rate {
            let amount = rhs.convert_amount(self.rate)?;
            Ok(self.amount / amount)
        } else {
            let amount = self.convert_amount(rhs.rate)?;
            Ok(amount / rhs.amount)
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

    /// indicate the flow is more than enough
    fn is_more_than_enough(&self) -> bool;

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

#[cfg(test)]
pub mod test {
    use crate::app::resources::resource_flow::ResourceFlow;
    use crate::app::resources::test::{setup_resource_a, setup_resource_b};
    use crate::app::resources::{RatePer, ResourceDefinition};

    pub(crate) struct TestInfo {
        pub flow: ResourceFlow<usize, f32>,
        pub resource: ResourceDefinition,
        pub amount_per_cycle: usize,
        pub amount: f32,
        pub rate: RatePer,
    }

    pub(crate) fn setup_flow_resource_a(amount: Option<usize>) -> TestInfo {
        let resource = setup_resource_a();
        let amount_per_cycle = amount.unwrap_or(2);
        let amount = amount_per_cycle as f32;
        let rate = RatePer::Minute;

        TestInfo {
            flow: ResourceFlow::new(&resource, amount_per_cycle, amount, rate),
            resource,
            amount_per_cycle,
            amount,
            rate,
        }
    }

    pub(crate) fn setup_flow_resource_b(amount: Option<usize>, rate: Option<RatePer>) -> TestInfo {
        let resource = setup_resource_b();
        let amount_per_cycle = amount.unwrap_or(1);
        let rate = rate.unwrap_or(RatePer::Minute);
        let amount = amount_per_cycle as f32;

        TestInfo {
            flow: ResourceFlow::new(&resource, amount_per_cycle, amount, rate),
            resource,
            amount_per_cycle,
            amount,
            rate,
        }
    }

    pub(crate) fn setup_flow_resource(
        resource: ResourceDefinition,
        amount: usize,
        rate: RatePer,
    ) -> TestInfo {
        let amount_per_cycle = amount;
        let amount = amount_per_cycle as f32;

        TestInfo {
            flow: ResourceFlow::new(&resource, amount_per_cycle, amount, rate),
            resource,
            amount_per_cycle,
            amount,
            rate,
        }
    }

    //-------------------Tests-------------------
}
