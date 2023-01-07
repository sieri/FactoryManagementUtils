use crate::recipe_window::{BasicRecipeWindowDescriptor, RecipeWindowGUI};
use crate::tests::test_framework as t;
use crate::tests::test_framework::TestResult;
use std::fmt::Write;
const EMPTY_WINDOW_TITLE: &str = "Test Window Empty";

fn setup_basic_recipe_window_empty() -> BasicRecipeWindowDescriptor {
    BasicRecipeWindowDescriptor::new(EMPTY_WINDOW_TITLE.to_string())
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
fn test_tooltip_empty() -> TestResult {
    let sample_window = setup_basic_recipe_window_empty();
    let expected = build_tooltip(
        [
            "Test Window Empty",
            "Inputs:|Outputs:            ",
            "       |Test Window Empty: 1",
            "       |               1.00/s",
        ]
        .as_slice(),
    );
    test_tooltip(sample_window, expected)
}
