use crate::app::{CommonManager, CoordinatesInfo};
use crate::resources::ManageFlow::{RecipeInput, RecipeOutput};
use crate::resources::{
    FlowError, FlowErrorType, ManageFlow, ManageResourceFlow, RatePer, RecipeInputResource,
    RecipeOutputResource, ResourceDefinition, ResourceFlow, Unit,
};
use crate::utils::{Io, Number};
use egui::Widget;
use std::default::Default;
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
        egui::Id::new(name + &*format!("{timestamp}"))
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
    inputs: Vec<ManageFlow<usize>>,

    ///list of outputs
    outputs: Vec<ManageFlow<usize>>,

    ///power used per cycle
    power: Option<ManageFlow<usize>>,

    ///Resource adding windows
    resource_adding_windows: Vec<ResourceAddingWindow<usize>>,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,
}

impl Default for BasicRecipeWindowDescriptor {
    fn default() -> Self {
        Self::new(String::from("Basic Recipe Window"))
    }
}

impl RecipeWindowGUI for BasicRecipeWindowDescriptor {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();

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
        self.window_coordinate.window = inner_response.response.rect;

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

        if open {
            commons
                .window_coordinates
                .insert(self.id, self.window_coordinate.clone());
        } else {
            commons.window_coordinates.remove(&self.id);
        }

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
        let output = RecipeOutput(RecipeOutputResource::new(resource, flow));
        Self {
            title,
            id,
            inputs: vec![],
            outputs: vec![output],
            power: None,
            resource_adding_windows: vec![],
            window_coordinate: CoordinatesInfo::default(),
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
        let resource_flow: &dyn ManageResourceFlow<usize> = match dir {
            Io::Input => match &self.inputs[resource_flow_index] {
                RecipeInput(r) => r,
                RecipeOutput(_) => {
                    panic!("Error!!! Impossible situation")
                }
            },
            Io::Output => match &self.outputs[resource_flow_index] {
                RecipeInput(_) => {
                    panic!("Error!!! Impossible situation")
                }
                RecipeOutput(r) => r,
            },
        };
        let resource = resource_flow.resource();
        let mut name = resource.clone().name;
        let name_len = name.len();
        let resource_flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let mut amount = resource_flow.amount;
        ui.horizontal(|ui| {
            match dir {
                Io::Input => {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.in_flow.push(btn_resp.rect);

                    if btn_resp.clicked() && commons.arrow_active {
                        commons.clicked_place_arrow_info =
                            Some((resource.clone(), self.id, resource_flow_index));
                    }
                }
                Io::Output => {}
            }

            egui::TextEdit::singleline(&mut name)
                .desired_width((name_len * 7) as f32)
                .show(ui);
            ui.label(":");
            egui::DragValue::new(&mut amount).ui(ui);
            ui.label("per cycle");

            match dir {
                Io::Input => {}
                Io::Output => {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.out_flow.push(btn_resp.rect);

                    if btn_resp.clicked() {
                        commons.clicked_start_arrow_info = Some((
                            resource.clone(),
                            self.id,
                            ui.layer_id(),
                            resource_flow_index,
                        ));
                    }
                }
            }
        });
    }

    fn show_power(&mut self, ui: &mut egui::Ui, _enabled: bool) {
        let power: &dyn ManageResourceFlow<usize> = match &self.power {
            None => {
                if ui.button("Add Power").clicked() {}
                return;
            }
            Some(a) => match a {
                RecipeInput(p) => p,
                RecipeOutput(p) => p,
            },
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

    pub(crate) fn get_resource(&self) -> ManageFlow<T> {
        let resource = ResourceDefinition {
            name: self.resource_name.clone(),
            unit: Unit::Piece,
        };
        let flow = ResourceFlow::new(&resource, self.amount_per_cycle, RatePer::Second);
        match self.dir {
            Io::Input => RecipeInput(RecipeInputResource::new(resource, flow)),
            Io::Output => RecipeOutput(RecipeOutputResource::new(resource, flow)),
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
pub(crate) enum ArrowUsageState {
    Active,
    Anchored,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ArrowFlow {
    pub(crate) id: egui::Id,
    pub(crate) state: ArrowUsageState,

    pub(crate) resource: ResourceDefinition,

    pub(crate) start_flow_window: egui::Id,
    pub(crate) end_flow_window: Option<egui::Id>,
    pub(crate) start_flow_index: usize,
    pub(crate) end_flow_index: usize,

    pub(crate) layer_id: egui::LayerId,
}

impl RecipeWindowGUI for ArrowFlow {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        let painter = ctx.layer_painter(self.layer_id);

        let start_coordinate_info = commons.window_coordinates.get(&self.start_flow_window);
        let start_point = match start_coordinate_info {
            None => return false,
            Some(r) => {
                let start_rect = r.window;
                let flow_rect = r.out_flow.get(self.start_flow_index);
                match flow_rect {
                    None => egui::Pos2 {
                        x: start_rect.max.x,
                        y: (start_rect.max.y - start_rect.min.y) / 2.0 + start_rect.min.y,
                    },
                    Some(rect) => egui::Pos2 {
                        x: start_rect.max.x,
                        y: (rect.max.y - rect.min.y) / 2.0 + rect.min.y,
                    },
                }
            }
        };
        let end_point = match self.state {
            ArrowUsageState::Active => ctx
                .pointer_hover_pos()
                .unwrap_or(egui::Pos2::new(10.0, 10.0)),
            ArrowUsageState::Anchored => {
                let end_rect = commons
                    .window_coordinates
                    .get(self.end_flow_window.as_ref().unwrap());
                match end_rect {
                    None => return false,
                    Some(r) => {
                        let start_rect = r.window;
                        let flow_rect = r.in_flow.get(self.end_flow_index);
                        match flow_rect {
                            None => egui::Pos2 {
                                x: start_rect.min.x,
                                y: (start_rect.max.y - start_rect.min.y) / 2.0 + start_rect.min.y,
                            },
                            Some(rect) => egui::Pos2 {
                                x: start_rect.min.x,
                                y: (rect.max.y - rect.min.y) / 2.0 + rect.min.y,
                            },
                        }
                    }
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
    pub(crate) fn new(
        resource: ResourceDefinition,
        start_flow: egui::Id,
        layer_id: egui::LayerId,
        flow_index: usize,
    ) -> Self {
        ArrowFlow {
            id: Self::gen_id(format!("Flow{start_flow:?}")),
            state: ArrowUsageState::Active,
            resource,
            start_flow_window: start_flow,
            end_flow_window: None,
            start_flow_index: flow_index,
            end_flow_index: 0,
            layer_id,
        }
    }

    pub(crate) fn put_end(
        &mut self,
        resource: ResourceDefinition,
        end_flow: egui::Id,
        flow_index: usize,
    ) -> Result<(), FlowError> {
        if resource != self.resource {
            return Err(FlowError::new(FlowErrorType::WrongResourceType));
        }

        self.end_flow_window = Some(end_flow);
        self.end_flow_index = flow_index;
        self.state = ArrowUsageState::Anchored;

        Ok(())
    }
}
