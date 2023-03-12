use crate::app::commons::CommonsManager;
use crate::app::resources::RatePer;
use std::f32;

pub(crate) mod arrow_flow;
pub(crate) mod base_recipe_window;
pub(crate) mod compound_recipe_window;
pub(crate) mod resource_adding_window;
pub(crate) mod resource_sink;
pub(crate) mod resources_sources;
pub(crate) mod simple_recipe_window;

pub trait RecipeWindowGUI {
    /// Show a recipe window on the frame
    ///
    /// # Arguments
    ///
    /// * `ctx`: the context it will spawn on
    /// * `enabled`: flag indicating it's enabled
    ///
    /// returns: `bool` flag if the window is still alive
    ///
    fn show(&mut self, commons: &mut CommonsManager, ctx: &egui::Context, enabled: bool) -> bool;

    fn generate_tooltip(&self) -> Result<String, std::fmt::Error>;
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone, Debug)]
pub enum RecipeWindowType {
    SimpleRecipe,
    CompoundRecipe,
    Source,
    Sink,
}

fn rate_combo(ui: &mut egui::Ui, rate: &mut RatePer) {
    egui::ComboBox::from_label("Time unit")
        .selected_text(format!("{rate:?}"))
        .show_ui(ui, |ui| {
            ui.selectable_value(rate, RatePer::Tick, "Tick");
            ui.selectable_value(rate, RatePer::Second, "Second");
            ui.selectable_value(rate, RatePer::Minute, "Minute");
            ui.selectable_value(rate, RatePer::Hour, "Hour");
        });
}

fn text_edit(ui: &mut egui::Ui, text: &mut String) {
    let text_len = text.len();
    egui::TextEdit::singleline(text)
        .desired_width((text_len * 7) as f32)
        .show(ui);
}

#[cfg(test)]
pub mod test {
    use crate::app::resources::recipe_input_resource::RecipeInputResource;
    use crate::app::resources::recipe_output_resource::RecipeOutputResource;
    use crate::app::resources::resource_flow::test::{
        setup_flow_resource_a, setup_flow_resource_b,
    };
    use crate::app::resources::test::setup_resource_a;
    use crate::app::resources::{resource_flow, ManageFlow, RatePer};
    use std::fmt::Write;

    pub(crate) struct ManageFlowTestInfo {
        pub manage_flow: ManageFlow<usize>,
        pub flow: resource_flow::test::TestInfo,
    }
    #[deprecated]
    pub(crate) fn setup_resource_a_input(amount: Option<usize>) -> ManageFlowTestInfo {
        let flow = setup_flow_resource_a(amount);
        let manage_flow = ManageFlow::RecipeInput(RecipeInputResource::new(
            setup_resource_a(),
            flow.flow.clone(),
        ));

        ManageFlowTestInfo { manage_flow, flow }
    }
    #[deprecated]
    pub(crate) fn setup_resource_a_output(amount: Option<usize>) -> ManageFlowTestInfo {
        let flow = setup_flow_resource_a(amount);
        let manage_flow = ManageFlow::RecipeOutput(RecipeOutputResource::new(
            flow.flow.resource.clone(),
            flow.flow.clone(),
        ));

        ManageFlowTestInfo { manage_flow, flow }
    }

    #[deprecated]
    pub(crate) fn setup_resource_b_input(
        amount: Option<usize>,
        rate: Option<RatePer>,
    ) -> ManageFlowTestInfo {
        let flow = setup_flow_resource_b(amount, rate);
        let manage_flow = ManageFlow::RecipeInput(RecipeInputResource::new(
            flow.flow.resource.clone(),
            flow.flow.clone(),
        ));

        ManageFlowTestInfo { manage_flow, flow }
    }
    #[deprecated]
    pub(crate) fn setup_resource_b_output(amount: Option<usize>) -> ManageFlowTestInfo {
        let flow = setup_flow_resource_b(amount, None);
        let manage_flow = ManageFlow::RecipeOutput(RecipeOutputResource::new(
            flow.flow.resource.clone(),
            flow.flow.clone(),
        ));

        ManageFlowTestInfo { manage_flow, flow }
    }

    pub(crate) fn setup_resource_output(flow: resource_flow::test::TestInfo) -> ManageFlowTestInfo {
        let manage_flow = ManageFlow::RecipeOutput(RecipeOutputResource::new(
            flow.resource.clone(),
            flow.flow.clone(),
        ));
        ManageFlowTestInfo { manage_flow, flow }
    }

    pub(crate) fn setup_resource_input(flow: resource_flow::test::TestInfo) -> ManageFlowTestInfo {
        let manage_flow = ManageFlow::RecipeInput(RecipeInputResource::new(
            flow.resource.clone(),
            flow.flow.clone(),
        ));
        ManageFlowTestInfo { manage_flow, flow }
    }

    ///Build to tooltip by concatenate lines together from an array
    pub(crate) fn build_tooltip(lines: &[&str]) -> String {
        let mut r = String::new();
        for l in lines {
            writeln!(r, "{l}").expect("Built tooltip failed");
        }
        r
    }
}
