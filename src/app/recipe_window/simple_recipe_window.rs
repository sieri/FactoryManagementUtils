use crate::app::commons::CommonsManager;
use crate::app::error::ShowError;
use crate::app::recipe_window::base_recipe_window::{
    BaseRecipeWindow, ConfigFeatures, RecipeWindowUser,
};
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};

use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ManageFlow;
use crate::utils::Io;
use egui::Context;
use log::{debug, trace};
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
                RecipeWindowType::SimpleRecipe,
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

    fn internal_calculation(&mut self) {
        trace!("[START] internal calculation for simple recipe window");
        let mut min_rate = 1.0f32;
        for input in self.inner_recipe.inputs.iter() {
            match input {
                ManageFlow::RecipeInput(input) => {
                    debug!("Resource: {}", input.resource().name);
                    debug!(
                        "inputs! {}{}",
                        input.total_in().amount,
                        input.total_in().rate.to_shortened_string()
                    );
                    debug!(
                        "outputs! {}{}",
                        input.total_out().amount,
                        input.total_out().rate.to_shortened_string()
                    );
                    let rate = (input.total_in().amount / input.total_out().amount).min(1.0);
                    min_rate = min_rate.min(rate);
                }
                ManageFlow::RecipeOutput(_) => {
                    self.inner_recipe.errors.push(ShowError::new(
                        "An input made its way to an output, this isn't possible".to_string(),
                    ));
                }
            }
        }
        trace!("[END] internal calculation for simple recipe window");
        let res = self.inner_recipe.update_flow(Io::Output);
        if let Err(e) = res {
            self.push_errors(ShowError::new(e.str()))
        }

        if min_rate < 1.0 {
            for output in self.inner_recipe.outputs.iter_mut() {
                match output {
                    ManageFlow::RecipeInput(_) => {
                        self.inner_recipe.errors.push(ShowError::new(
                            "An outputs made its way to an input, this isn't possible".to_string(),
                        ));
                    }
                    ManageFlow::RecipeOutput(output) => {
                        output.created.amount *= min_rate;
                    }
                }
            }
        }
    }

    fn back_propagation_internal_calculation(
        &mut self,
        rate: f32,
        _amount: Option<ResourceFlow<usize, f32>>,
    ) {
        trace!("[START] back propagation internal calculation for simple recipe window");
        for input in self.inner_recipe.inputs.iter_mut() {
            match input {
                ManageFlow::RecipeInput(input) => {
                    debug!("input needed before back propagation={}", input.needed);
                    input.needed.amount *= rate;
                    debug!("input needed after back propagation={}", input.needed);
                }
                ManageFlow::RecipeOutput(_) => {
                    self.inner_recipe.errors.push(ShowError::new(
                        "An input made its way to an output, this isn't possible".to_string(),
                    ));
                }
            }
        }
        trace!("[END] back propagation internal calculation for simple recipe window");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::app::recipe_window;
    use crate::app::recipe_window::base_recipe_window::tests::RecipeResourceInfos;
    use crate::app::recipe_window::base_recipe_window::{
        BaseRecipeWindow, ConfigFeatures, RecipeWindowUser,
    };
    use crate::app::recipe_window::simple_recipe_window::SimpleRecipeWindow;
    use crate::app::recipe_window::test::{
        setup_resource_a_input, setup_resource_b_input, setup_resource_b_output,
    };
    use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
    use crate::app::resources::test::setup_resource_a;
    use crate::app::resources::{RatePer, ResourceDefinition, Unit};
    use crate::utils::test_env;

    use crate::app::resources::resource_flow::ResourceFlow;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    impl SimpleRecipeWindow {
        pub fn new_with_custom_output(flow: ResourceFlow<usize, f32>) -> Self {
            let inner_recipe = BaseRecipeWindow::new_with_custom_output(
                ConfigFeatures {
                    interactive_input: true,
                    pure_time_input: false,
                    interactive_output: true,
                    pure_time_output: false,
                    show_power: true,
                    show_time: true,
                },
                RecipeWindowType::SimpleRecipe,
                flow,
            );

            Self { inner_recipe }
        }
    }

    pub(crate) struct TestInfo {
        pub recipe: SimpleRecipeWindow,
        pub input_resources: Vec<RecipeResourceInfos>,
        pub output_resources: Vec<RecipeResourceInfos>,
    }

    pub(crate) fn setup_basic_recipe_window_empty() -> TestInfo {
        let title = "Test Window Empty";
        TestInfo {
            recipe: SimpleRecipeWindow::new(title.to_string()),
            output_resources: vec![RecipeResourceInfos {
                def: ResourceDefinition {
                    name: title.to_string(),
                    unit: Unit::Piece,
                },
                amount: 1.0,
                amount_per_cycle: 1,
                rate: RatePer::Second,
            }],

            input_resources: vec![],
        }
    }

    #[deprecated]
    pub(crate) fn setup_simple_recipe_one_to_one() -> TestInfo {
        let title = "Test Window One To One";
        let mut w = SimpleRecipeWindow::new(title.to_string());
        let resource_a = setup_resource_a_input(None);
        w.inner_recipe.inputs.push(resource_a.manage_flow);
        TestInfo {
            recipe: w,
            output_resources: vec![RecipeResourceInfos {
                def: ResourceDefinition {
                    name: title.to_string(),
                    unit: Unit::Piece,
                },
                amount: 1.0,
                amount_per_cycle: 1,
                rate: RatePer::Second,
            }],

            input_resources: vec![RecipeResourceInfos {
                def: resource_a.flow.resource,
                amount: resource_a.flow.amount,
                amount_per_cycle: resource_a.flow.amount_per_cycle,
                rate: resource_a.flow.rate,
            }],
        }
    }

    #[deprecated]
    pub(crate) fn setup_simple_recipe_one_to_one_b() -> TestInfo {
        let resource_a = setup_resource_a();
        let _title = resource_a.name.clone();
        let mut w = SimpleRecipeWindow::new_with_custom_output(ResourceFlow::new(
            &resource_a,
            3,
            3.0,
            RatePer::Second,
        ));

        let rate = RatePer::Minute;
        let amount = 8;
        let resource_b = setup_resource_b_input(Some(amount), Some(rate));
        w.inner_recipe.inputs.push(resource_b.manage_flow);

        TestInfo {
            recipe: w,
            output_resources: vec![RecipeResourceInfos {
                def: resource_a,
                amount: amount as f32,
                amount_per_cycle: amount,
                rate,
            }],

            input_resources: vec![RecipeResourceInfos {
                def: resource_b.flow.resource,
                amount: resource_b.flow.amount,
                amount_per_cycle: resource_b.flow.amount_per_cycle,
                rate: resource_b.flow.rate,
            }],
        }
    }

    pub(crate) fn setup_simple_recipe_one_to_one_custom(
        input: Option<recipe_window::test::ManageFlowTestInfo>,
        output: Option<recipe_window::test::ManageFlowTestInfo>,
    ) -> TestInfo {
        let input = input.unwrap_or_else(|| setup_resource_a_input(None));
        let output = output.unwrap_or_else(|| setup_resource_b_output(None));

        let _title = output.flow.resource.name.clone();
        let mut w = SimpleRecipeWindow::new_with_custom_output(output.flow.flow.clone());

        w.inner_recipe.inputs.push(input.manage_flow);

        let input_test_info = input.flow;
        let output_test_info = output.flow;

        TestInfo {
            recipe: w,
            output_resources: vec![RecipeResourceInfos {
                def: output_test_info.resource,
                amount: output_test_info.amount,
                amount_per_cycle: output_test_info.amount_per_cycle,
                rate: output_test_info.rate,
            }],
            input_resources: vec![RecipeResourceInfos {
                def: input_test_info.resource,
                amount: input_test_info.amount,
                amount_per_cycle: input_test_info.amount_per_cycle,
                rate: input_test_info.rate,
            }],
        }
    }

    pub(crate) fn setup_list_of_window() -> [TestInfo; 3] {
        [
            setup_basic_recipe_window_empty(),
            setup_simple_recipe_one_to_one(),
            setup_simple_recipe_one_to_one_b(),
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
        test_env::setup();
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
        test_env::setup();
        let sample_window = setup_simple_recipe_one_to_one();
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
        test_env::setup();
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
        test_env::setup();
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
