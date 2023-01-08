use crate::recipe_window::{BasicRecipeWindowDescriptor, RecipeWindowGUI};
use crate::resources::{ManageFlow, RecipeInputResource};
use crate::tests::resources_test::{setup_flow_resource_a, setup_resource_a};
use crate::tests::test_framework as t;
use crate::tests::test_framework::TestResult;
use std::fmt::Write;

pub(crate) fn setup_basic_recipe_window_empty() -> BasicRecipeWindowDescriptor {
    BasicRecipeWindowDescriptor::new("Test Window Empty".to_string())
}

pub(crate) fn setup_basic_recipe_one_to_one() -> BasicRecipeWindowDescriptor {
    let mut w = BasicRecipeWindowDescriptor::new("Test Window One To One".to_string());

    w.inputs.push(setup_resource_a_input());

    w
}

pub(crate) fn setup_resource_a_input() -> ManageFlow<usize> {
    ManageFlow::RecipeInput(RecipeInputResource::new(
        setup_resource_a(),
        setup_flow_resource_a(),
    ))
}

fn test_tooltip(window: BasicRecipeWindowDescriptor, expected: String) -> TestResult {
    t::assert_equal(
        expected,
        window.generate_tooltip().unwrap(),
        "Tooltip doesn't match",
    )
}

fn build_tooltip(lines: &[&str]) -> String {
    let mut r = String::new();
    for l in lines {
        write!(r, "{}\n", l).expect("Built tooltip failed");
    }
    r
}

//-------------------Tests-------------------

#[test]
#[ignore = "Not working"]
fn test_tooltip_empty() -> TestResult {
    let sample_window = setup_basic_recipe_window_empty();
    let expected = build_tooltip(
        [
            "Test Window Empty",
            "Inputs: |Outputs:            ",
            "        |Test Window Empty: 1",
            "        |               1.00/s",
        ]
        .as_slice(),
    );
    test_tooltip(sample_window, expected)
}

#[test]
#[ignore = "Not working"]
fn test_tooltip_one_to_one() -> TestResult {
    let sample_window = setup_basic_recipe_one_to_one();
    let expected = build_tooltip(
        [
            "Test Window One To One",
            "Inputs:      |Outputs:                 ",
            "Resource A: 2|Test Window One To One: 1",
            "     2.00/min|                   1.00/s",
        ]
        .as_slice(),
    );
    test_tooltip(sample_window, expected)
}
