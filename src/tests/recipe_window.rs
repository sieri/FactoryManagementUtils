use crate::app::resources::recipe_input_resource::RecipeInputResource;
use crate::app::resources::ManageFlow;
use crate::tests::resources::{setup_flow_resource_a, setup_resource_a};
use std::fmt::Write;

mod basic_recipe_window_descriptor;

pub(crate) fn setup_resource_a_input() -> ManageFlow<usize> {
    ManageFlow::RecipeInput(RecipeInputResource::new(
        setup_resource_a(),
        setup_flow_resource_a(),
    ))
}

///Build to tooltip by concatenate lines together from an array
fn build_tooltip(lines: &[&str]) -> String {
    let mut r = String::new();
    for l in lines {
        writeln!(r, "{l}").expect("Built tooltip failed");
    }
    r
}
