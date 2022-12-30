use crate::resources::{
    ManageResourceFlow, RatePer, RecipeInputResource, RecipeOutputResource, ResourceDefinition,
    ResourceFlow, Unit,
};
use crate::utils::{Io, Number};
use egui::Widget;
use std::time::{SystemTime, UNIX_EPOCH};

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
    fn show(&mut self, ctx: &egui::Context, enabled: bool) -> bool;

    fn gen_id(name: String) -> egui::Id {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        egui::Id::new(name + &*format!("{}", timestamp))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Descriptor for a Basic Recipe window, the recipe is directly calculated
pub struct BasicRecipeWindowDescriptor {
    ///Title of the recipe
    title: String,

    ///unique id of the recipe
    id: egui::Id,

    ///list of inputs
    #[serde(skip)] //TODO: Serialize this probably manually
    inputs: Vec<Box<dyn ManageResourceFlow<usize>>>,

    ///list of outputs
    #[serde(skip)] //TODO: Serialize this probably manually
    outputs: Vec<Box<dyn ManageResourceFlow<usize>>>,

    ///power used per cycle
    #[serde(skip)] //TODO: Serialize this probably manually
    power: Option<Box<dyn ManageResourceFlow<usize>>>,

    ///Resource adding windows
    resource_adding_windows: Vec<ResourceAddingWindow<usize>>,
}

impl Default for BasicRecipeWindowDescriptor {
    fn default() -> Self {
        Self::new(String::from("Basic Recipe Window"))
    }
}

impl RecipeWindowGUI for BasicRecipeWindowDescriptor {
    fn show(&mut self, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        self.show_inputs(ui, enabled);
                        self.show_outputs(ui, enabled);
                    });
                })
            });

        self.resource_adding_windows.retain_mut(|window| {
            let open = window.show(ctx, enabled);
            if window.okay {
                //add the response
                let resource = window.get_resource();
                match window.dir {
                    Io::Input => {
                        self.inputs.push(resource);
                    }
                    Io::Output => {
                        self.outputs.push(resource);
                    }
                };
            }
            open && !window.okay
        });

        open
    }
}

impl BasicRecipeWindowDescriptor {
    /// Create a new basic recipe
    ///
    /// # Arguments
    ///
    /// * `title`: Title of the recipe
    ///
    /// returns: BasicRecipeWindowDescriptor
    pub fn new(title: String) -> Self {
        let id = BasicRecipeWindowDescriptor::gen_id(title.clone());
        Self {
            title,
            id,
            inputs: vec![],
            outputs: vec![],
            power: None,
            resource_adding_windows: vec![],
        }
    }

    fn show_inputs(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("inputs");

                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(egui::RichText::new("➕").color(egui::Rgba::GREEN)),
                    )
                    .clicked()
                {
                    self.open_resource_adding_window(Io::Input);
                }
            });
            for flow in self.inputs.iter() {
                self.show_flow(flow, Io::Input, ui, enabled);
            }
        });
    }

    fn show_outputs(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("outputs");
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(egui::RichText::new("➕").color(egui::Rgba::GREEN)),
                    )
                    .clicked()
                {
                    self.open_resource_adding_window(Io::Output);
                }
            });
            for flow in self.outputs.iter() {
                self.show_flow(flow, Io::Output, ui, enabled);
            }
        });
    }

    #[allow(clippy::borrowed_box)]
    fn show_flow(
        &self,
        resource_flow: &Box<dyn ManageResourceFlow<usize>>,
        dir: Io,
        ui: &mut egui::Ui,
        _enabled: bool,
    ) {
        //get variables
        let resource = resource_flow.resource();
        let mut name = resource.name;
        let resource_flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let mut amount = resource_flow.amount;
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut name);
            ui.label(":");
            egui::DragValue::new(&mut amount).ui(ui);
            ui.label("per cycle");
        });
    }

    fn open_resource_adding_window(&mut self, dir: Io) {
        let title = format!(
            "Resource for {}{}",
            self.title,
            self.resource_adding_windows.len() + 1
        );
        let window = ResourceAddingWindow::<usize>::new(title, dir);
        self.resource_adding_windows.push(window);
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ResourceAddingWindow<T> {
    ///Title
    pub(crate) title: String,

    ///private id
    id: egui::Id,

    ///Resource Name
    pub(crate) resource_name: String,

    ///Amount per cycle
    pub(crate) amount_per_cycle: T,

    ///Amount per unit of time
    pub(crate) amount_per_time: T,

    ///Rate
    pub(crate) rate: RatePer,

    ///Io direction
    pub(crate) dir: Io,

    ///flag indicating if validated
    pub(crate) okay: bool,
}

impl<T: Number> ResourceAddingWindow<T> {
    pub fn new(title: String, dir: Io) -> Self {
        Self {
            title: title.clone(),
            id: Self::gen_id(title),
            resource_name: "".to_string(),
            amount_per_cycle: T::one(),
            amount_per_time: T::one(),
            rate: RatePer::Second,
            dir,
            okay: false,
        }
    }

    pub(crate) fn get_resource(&self) -> Box<dyn ManageResourceFlow<T>> {
        let resource = ResourceDefinition {
            name: self.resource_name.clone(),
            unit: Unit::Piece,
        };
        let flow = ResourceFlow::new(&resource, self.amount_per_cycle, RatePer::Second);
        match self.dir {
            Io::Input => Box::new(RecipeInputResource::new(resource, flow)),
            Io::Output => Box::new(RecipeOutputResource::new(resource, flow)),
        }
    }
}

impl<T: Number> RecipeWindowGUI for ResourceAddingWindow<T> {
    fn show(&mut self, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        egui::TextEdit::singleline(&mut self.resource_name)
                            .hint_text("resource name")
                            .show(ui);
                        ui.label("Amount:");

                        //ui.horizontal( |ui| {
                        egui::DragValue::new(&mut self.amount_per_cycle).ui(ui);
                        ui.label("per cycle");
                        // });
                        //TODO Implement time based entering
                        //egui::DragValue::new(&mut self.amount_per_time).ui(ui);
                    });
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let button = ui.button(match self.dir {
                            Io::Input => "Add input",
                            Io::Output => "Add output",
                        });
                        if button.clicked() {
                            self.okay = true
                        }
                    });
                });
            });

        open
    }
}
