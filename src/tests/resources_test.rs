use crate::resources::{RatePer, ResourceDefinition, ResourceFlow, Unit};

pub(crate) fn setup_resource_a() -> ResourceDefinition {
    ResourceDefinition {
        name: "Resource A".to_string(),
        unit: Unit::Piece,
    }
}

pub(crate) fn setup_flow_resource_a() -> ResourceFlow<usize, f32> {
    ResourceFlow::new(&setup_resource_a(), 2, 2.0, RatePer::Minute)
}

//-------------------Tests-------------------
