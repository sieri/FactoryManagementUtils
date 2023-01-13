use crate::app::commons::CommonsManager;
use crate::app::error::ShowError;
use crate::app::recipe_window::base_recipe_window::{
    BaseRecipeWindow, ConfigFeatures, RecipeWindowUser,
};
use crate::app::recipe_window::RecipeWindowGUI;

use egui::Context;
use std::fmt::Error;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct SimpleRecipeWindow {
    pub inner_recipe: BaseRecipeWindow,
}

impl SimpleRecipeWindow {
    pub fn new(title: String) -> Self {
        Self {
            inner_recipe: BaseRecipeWindow::new(
                title,
                ConfigFeatures {
                    interactive_input: true,
                    pure_time_input: false,
                    interactive_output: true,
                    pure_time_output: false,
                    show_power: true,
                    show_time: true,
                },
            ),
        }
    }
}

impl RecipeWindowGUI for SimpleRecipeWindow {
    fn show(&mut self, commons: &mut CommonsManager, ctx: &Context, enabled: bool) -> bool {
        let mut open = true;
        self.inner_recipe.clean_coordinates();

        let title = self.inner_recipe.gen_title_string();
        let response = self
            .inner_recipe
            .window(commons, ctx, enabled, &mut open, title);

        let inner_response = response.unwrap();
        self.inner_recipe.update_coordinates(&inner_response);
        self.inner_recipe
            .show_tooltips(commons, ctx, inner_response);

        self.inner_recipe
            .show_resource_adding_windows(commons, ctx, enabled);

        self.inner_recipe.push_errors(commons);

        self.inner_recipe.push_coordinates(commons, &mut open);

        open
    }

    fn generate_tooltip(&self) -> Result<String, Error> {
        self.inner_recipe.generate_tooltip()
    }
}

impl RecipeWindowUser<'static> for SimpleRecipeWindow {
    type Gen = Self;

    fn recipe(self) -> BaseRecipeWindow {
        self.inner_recipe
    }

    fn push_errors(&mut self, e: ShowError) {
        self.inner_recipe.errors.push(e);
    }

    fn gen_ids(&mut self) {
        self.inner_recipe.gen_ids();
    }
}

#[cfg(test)]
pub mod tests {
    use crate::app::recipe_window;
    use crate::app::recipe_window::base_recipe_window::tests::RecipeResourceInfos;
    use crate::app::recipe_window::base_recipe_window::RecipeWindowUser;
    use crate::app::recipe_window::simple_recipe_window::SimpleRecipeWindow;
    use crate::app::recipe_window::test::setup_resource_a_input;
    use crate::app::recipe_window::RecipeWindowGUI;
    use crate::app::resources::{RatePer, ResourceDefinition, Unit};
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    pub(crate) struct TestInfo {
        pub recipe: SimpleRecipeWindow,
        pub input_resource: Vec<RecipeResourceInfos>,
        pub output_resource: Vec<RecipeResourceInfos>,
    }

    pub(crate) fn setup_basic_recipe_window_empty() -> TestInfo {
        let title = "Test Window Empty";
        TestInfo {
            recipe: SimpleRecipeWindow::new(title.to_string()),
            output_resource: vec![RecipeResourceInfos {
                def: ResourceDefinition {
                    name: title.to_string(),
                    unit: Unit::Piece,
                },
                amount: 1.0,
                amount_per_cycle: 1,
                rate: RatePer::Second,
            }],

            input_resource: vec![],
        }
    }

    pub(crate) fn setup_basic_recipe_one_to_one() -> TestInfo {
        let title = "Test Window One To One";
        let mut w = SimpleRecipeWindow::new(title.to_string());
        let resource_a = setup_resource_a_input();
        w.inner_recipe.inputs.push(resource_a.manage_flow);

        TestInfo {
            recipe: w,
            output_resource: vec![RecipeResourceInfos {
                def: ResourceDefinition {
                    name: title.to_string(),
                    unit: Unit::Piece,
                },
                amount: 1.0,
                amount_per_cycle: 1,
                rate: RatePer::Second,
            }],

            input_resource: vec![RecipeResourceInfos {
                def: resource_a.flow.resource,
                amount: resource_a.flow.amount,
                amount_per_cycle: resource_a.flow.amount_per_cycle,
                rate: resource_a.flow.rate,
            }],
        }
    }

    pub(crate) fn setup_list_of_window() -> [TestInfo; 2] {
        [
            setup_basic_recipe_window_empty(),
            setup_basic_recipe_one_to_one(),
        ]
    }

    fn perform_test_tooltip(window: SimpleRecipeWindow, expected: String) {
        assert_eq!(
            expected,
            window.generate_tooltip().unwrap(),
            "Tooltip doesn't match",
        );
    }

    //-------------------Tests-------------------

    #[test]
    #[ignore = "Not working https://github.com/sieri/FactoryManagementUtils/issues/1"] //TODO: FIX
    fn test_tooltip_empty() {
        let sample_window = setup_basic_recipe_window_empty();
        let expected = recipe_window::test::build_tooltip(
            [
                "Test Window Empty",
                "Inputs: |Outputs:            ",
                "        |Test Window Empty: 1",
                "        |               1.00/s",
            ]
            .as_slice(),
        );
        perform_test_tooltip(sample_window.recipe, expected);
    }

    #[test]
    #[ignore = "Not working https://github.com/sieri/FactoryManagementUtils/issues/1"] //TODO: FIX
    fn test_tooltip_one_to_one() {
        let sample_window = setup_basic_recipe_one_to_one();
        let expected = recipe_window::test::build_tooltip(
            [
                "Test Window One To One",
                "Inputs:      |Outputs:                 ",
                "Resource A: 2|Test Window One To One: 1",
                "     2.00/min|                   1.00/s",
            ]
            .as_slice(),
        );
        perform_test_tooltip(sample_window.recipe, expected);
    }

    #[test]
    fn test_serialization() {
        let originals = setup_list_of_window();
        let mut strings = Vec::new();
        for original in originals.iter() {
            let recipe = &original.recipe;
            let mut vec = vec![0u8];
            let s = Cursor::new(&mut vec);
            let result = recipe.serialize(&mut serde_json::Serializer::new(s));

            if let Err(e) = result {
                panic!("serialization error {e}");
            }
            strings.push(vec)
        }

        let mut deserializes = Vec::new();
        for string in strings {
            let cursor = Cursor::new(&string);
            let mut des = serde_json::Deserializer::from_reader(cursor);
            let result = SimpleRecipeWindow::deserialize(&mut des);
            if let Err(e) = result {
                panic!("deserialization error {e}");
            }
            deserializes.push(result.unwrap());
        }

        for (original, deserialized) in originals.iter().zip(deserializes.iter()) {
            let recipe = &original.recipe;
            assert_eq!(recipe, deserialized, "Deserialization doesn't match");
        }
    }

    #[test]
    fn test_save_and_load() {
        let mut originals = setup_list_of_window();
        let mut saved = vec![];
        for original in originals.iter_mut() {
            let recipe = &mut original.recipe;
            saved.push(recipe.save().expect("Not saved"));
        }
        let mut load = vec![];
        for s in saved {
            load.push(SimpleRecipeWindow::load(s).expect("Not loaded"))
        }

        for (original, loaded) in originals.iter().zip(load.iter()) {
            let recipe = &original.recipe;
            assert!(
                recipe.inner_recipe.equivalent(&loaded.inner_recipe),
                "Original and loaded should be are not equivalent",
            );
            assert_ne!(
                recipe, loaded,
                "Original and loaded should be different in ids",
            );
        }
    }
}
