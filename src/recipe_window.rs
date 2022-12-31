use crate::app::CommonManager;
use crate::resources::{
    ManageResourceFlow, RatePer, RecipeInputResource, RecipeOutputResource, ResourceDefinition,
    ResourceFlow, Unit,
};
use crate::utils::{Io, Number};
use eframe::emath::Rect;
use egui::accesskit::AriaCurrent::False;
use egui::{Sense, Widget};
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
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool;

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

    ///Outgoing resource flows
    out_flow: Vec<ArrowFlow>,
}

impl Default for BasicRecipeWindowDescriptor {
    fn default() -> Self {
        Self::new(String::from("Basic Recipe Window"))
    }
}

impl RecipeWindowGUI for BasicRecipeWindowDescriptor {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;
        let response = egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        self.show_inputs(commons, ui, enabled);
                        ui.separator();
                        self.show_outputs(commons, ui, enabled);
                    });
                    ui.separator();
                    self.show_power(ui, enabled);
                })
            });
        let inner_response = response.unwrap();
        if open {
            commons
                .window_coordinates
                .insert(self.id, inner_response.response.rect);
            let resp = inner_response.response.interact(Sense::click());
            if resp.clicked() && commons.arrow_active {
                commons.clicked_place_arrow_id = Some(self.id);
            }
        } else {
            commons.window_coordinates.remove(&self.id);
        }
        self.resource_adding_windows.retain_mut(|window| {
            let open = window.show(commons, ctx, enabled);
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

        self.out_flow
            .retain_mut(|arrow| arrow.show(commons, ctx, enabled));

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
        let resource = ResourceDefinition {
            name: title.clone(),
            unit: Unit::Piece,
        };
        let flow = ResourceFlow::new(&resource, 1, RatePer::Second);
        let output = Box::new(RecipeOutputResource::new(resource, flow));
        Self {
            title,
            id,
            inputs: vec![],
            outputs: vec![output],
            power: None,
            resource_adding_windows: vec![],
            out_flow: vec![],
        }
    }

    fn show_inputs(&mut self, commons: &mut CommonManager, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Inputs");

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

            for i in 0..self.inputs.len() {
                self.show_flow(commons, i, Io::Input, ui, enabled);
            }
        });
    }

    fn show_outputs(&mut self, commons: &mut CommonManager, ui: &mut egui::Ui, enabled: bool) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Outputs");
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
            for i in 0..self.outputs.len() {
                self.show_flow(commons, i, Io::Output, ui, enabled);
            }
        });
    }

    #[allow(clippy::borrowed_box)]
    fn show_flow(
        &mut self,
        commons: &mut CommonManager,
        resource_flow_index: usize,
        dir: Io,
        ui: &mut egui::Ui,
        _enabled: bool,
    ) {
        //get variables
        let resource_flow = match dir {
            Io::Input => &self.inputs[resource_flow_index],
            Io::Output => &self.outputs[resource_flow_index],
        };
        let resource = resource_flow.resource();
        let mut name = resource.name;
        let name_len = name.len();
        let resource_flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let mut amount = resource_flow.amount;
        ui.horizontal(|ui| {
            egui::TextEdit::singleline(&mut name)
                .desired_width((name_len * 7) as f32)
                .show(ui);
            ui.label(":");
            egui::DragValue::new(&mut amount).ui(ui);
            ui.label("per cycle");

            match dir {
                Io::Input => {}
                Io::Output => {
                    if ui.button("⭕").clicked() {
                        commons.clicked_start_arrow_id = Some((self.id, ui.layer_id()));
                    }
                }
            }
        });
    }

    fn show_power(&mut self, ui: &mut egui::Ui, _enabled: bool) {
        let power = match &self.power {
            None => {
                if ui.button("Add Power").clicked() {}
                return;
            }
            Some(a) => a,
        };
        //get variables
        let resource = power.resource();
        let mut name = resource.name;
        let resource_flow = power.total_out();
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
    fn show(&mut self, _commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
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
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
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

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
enum ArrowUsageState {
    Active,
    Anchored,
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
pub struct ArrowFlow {
    id: egui::Id,

    state: ArrowUsageState,

    start_flow: egui::Id,
    end_flow: Option<egui::Id>,

    layer_id: egui::LayerId,
}

impl RecipeWindowGUI for ArrowFlow {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        let painter = ctx.layer_painter(self.layer_id);

        let start_rect = commons.window_coordinates.get(&self.start_flow);
        let start_point = match start_rect {
            None => return false,
            Some(r) => r.max,
        };

        let end_point = match self.state {
            ArrowUsageState::Active => ctx
                .pointer_hover_pos()
                .unwrap_or(egui::Pos2::new(10.0, 10.0)),
            ArrowUsageState::Anchored => {
                let end_rect = commons
                    .window_coordinates
                    .get(self.end_flow.as_ref().unwrap());
                match end_rect {
                    None => return false,
                    Some(r) => r.min,
                }
            }
        };

        let color = match enabled {
            true => egui::Color32::RED,
            false => egui::Color32::GRAY,
        };

        match self.state {
            ArrowUsageState::Active => {
                commons.arrow_active = true;
            }
            ArrowUsageState::Anchored => {}
        }

        painter.line_segment([start_point, end_point], egui::Stroke::new(5.0, color));

        true
    }
}

impl ArrowFlow {
    pub(crate) fn new(start_flow: egui::Id, layer_id: egui::LayerId) -> Self {
        ArrowFlow {
            id: Self::gen_id(format!("Flow{:?}", start_flow)),
            state: ArrowUsageState::Active,
            start_flow,
            end_flow: None,
            layer_id,
        }
    }

    pub(crate) fn put_end(&mut self, end_flow: egui::Id) {
        self.end_flow = Some(end_flow);
        self.state = ArrowUsageState::Anchored;
    }
}
