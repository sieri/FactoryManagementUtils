use crate::app::resources::{resource_flow, RatePer, ResourceDefinition, Unit};

pub(crate) fn setup_resource_a() -> ResourceDefinition {
    ResourceDefinition {
        name: "Resource A".to_string(),
        unit: Unit::Piece,
    }
}

pub(crate) fn setup_flow_resource_a() -> resource_flow<usize, f32> {
    resource_flow::new(&setup_resource_a(), 2, 2.0, RatePer::Minute)
}

//-------------------Tests-------------------
