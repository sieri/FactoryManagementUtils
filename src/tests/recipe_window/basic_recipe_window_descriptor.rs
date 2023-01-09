use crate::app::recipe_window::basic_recipe_window_descriptor::BasicRecipeWindowDescriptor;
use crate::app::recipe_window::RecipeWindowGUI;
use crate::tests::recipe_window::setup_resource_a_input;
use crate::tests::test_framework::{TestError, TestResult};
use crate::tests::{recipe_window, test_framework as t};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

pub(crate) fn setup_basic_recipe_window_empty() -> BasicRecipeWindowDescriptor {
    BasicRecipeWindowDescriptor::new("Test Window Empty".to_string())
}

pub(crate) fn setup_basic_recipe_one_to_one() -> BasicRecipeWindowDescriptor {
    let mut w = BasicRecipeWindowDescriptor::new("Test Window One To One".to_string());

    w.inputs.push(setup_resource_a_input());

    w
}

pub(crate) fn setup_list_of_window() -> [BasicRecipeWindowDescriptor; 2] {
    [
        setup_basic_recipe_window_empty(),
        setup_basic_recipe_one_to_one(),
    ]
}

fn test_tooltip(window: BasicRecipeWindowDescriptor, expected: String) -> TestResult {
    t::assert_equal(
        expected,
        window.generate_tooltip().unwrap(),
        "Tooltip doesn't match",
    )
}

//-------------------Tests-------------------

#[test]
#[ignore = "Not working https://github.com/sieri/FactoryManagementUtils/issues/1"] //TODO: FIX
fn test_tooltip_empty() -> TestResult {
    let sample_window = setup_basic_recipe_window_empty();
    let expected = recipe_window::build_tooltip(
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
#[ignore = "Not working https://github.com/sieri/FactoryManagementUtils/issues/1"] //TODO: FIX
fn test_tooltip_one_to_one() -> TestResult {
    let sample_window = setup_basic_recipe_one_to_one();
    let expected = recipe_window::build_tooltip(
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

#[test]
fn test_serialization() -> TestResult {
    let originals = setup_list_of_window();
    let mut strings = Vec::new();
    for original in originals.iter() {
        let mut vec = vec![0u8];
        let s = Cursor::new(&mut vec);
        let result = original.serialize(&mut serde_json::Serializer::new(s));

        if let Err(e) = result {
            return Err(TestError {
                text: format!("serialization error {e}"),
            });
        }
        strings.push(vec)
    }

    let mut deserializes = Vec::new();
    for string in strings {
        let cursor = Cursor::new(&string);
        let mut des = serde_json::Deserializer::from_reader(cursor);
        let result = BasicRecipeWindowDescriptor::deserialize(&mut des);
        if let Err(e) = result {
            return Err(TestError {
                text: format!("deserialization error {e}"),
            });
        }
        deserializes.push(result.unwrap());
    }

    for (original, deserialized) in originals.iter().zip(deserializes.iter()) {
        t::assert_equal(original, deserialized, "Deserialization doesn't match")?;
    }

    Ok(())
}

#[test]
fn test_save_and_load() -> TestResult {
    let mut originals = setup_list_of_window();
    let mut saved = vec![];
    for original in originals.iter_mut() {
        saved.push(original.save().expect("Not saved"));
    }
    let mut load = vec![];
    for s in saved {
        load.push(BasicRecipeWindowDescriptor::load(s).expect("Not loaded"))
    }

    for (original, loaded) in originals.iter().zip(load.iter()) {
        t::assert_custom(
            original,
            loaded,
            "Original and loaded should be are not equivalent",
            |a, b| a.equivalent(b),
        )?;
        t::assert_not_equal(
            original,
            loaded,
            "Original and loaded should be different in ids",
        )?;
    }

    Ok(())
}
