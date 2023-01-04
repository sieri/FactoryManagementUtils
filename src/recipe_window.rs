use crate::app::{CommonManager, CoordinatesInfo, ShowError};
use crate::resources::ManageFlow::{RecipeInput, RecipeOutput};
use crate::resources::{
    FlowError, FlowErrorType, ManageFlow, ManageResourceFlow, RatePer, RecipeInputResource,
    RecipeOutputResource, ResourceDefinition, ResourceFlow, Unit,
};
use crate::utils::{Io, Number};
use egui::Widget;
use std::default::Default;
use std::f32;
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

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
pub enum RecipeWindowType {
    Basic,
    Source,
    Sink,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Descriptor for a Basic Recipe window, the recipe is directly calculated
pub struct BasicRecipeWindowDescriptor {
    ///Title of the recipe
    title: String,

    ///unique id of the recipe
    pub(crate) id: egui::Id,

    ///list of inputs
    pub(crate) inputs: Vec<ManageFlow<usize>>,

    ///list of outputs
    pub(crate) outputs: Vec<ManageFlow<usize>>,

    ///power used per cycle
    power: Option<ManageFlow<usize>>,

    ///Resource adding windows
    resource_adding_windows: Vec<ResourceAddingWindow<usize>>,

    ///Time of cycle
    time_cycle: usize,

    ///Time unit of cycle
    time_unit: RatePer,

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
                    let result = self.show_time_settings(ui, enabled);
                    if result.is_err() {
                        commons.add_error(ShowError::new(format!("{}", result.err().unwrap())));
                    }
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
        let mut flow = ResourceFlow::new(&resource, 1, 1.0f32, RatePer::Second);
        let _ = flow.convert_time_base(1, RatePer::Second);
        let output = RecipeOutput(RecipeOutputResource::new(resource, flow));
        Self {
            title,
            id,
            inputs: vec![],
            outputs: vec![output],
            power: None,
            resource_adding_windows: vec![],
            time_cycle: 1,
            time_unit: RatePer::Second,
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

        let color = match resource_flow.is_enough() {
            true => egui::Rgba::GREEN,
            false => egui::Rgba::RED,
        };

        let resource_flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let rate = resource_flow.rate;
        let mut amount = resource_flow.amount_per_cycle;
        let mut amount_per_time = resource_flow.amount;

        ui.horizontal(|ui| {
            match dir {
                Io::Input => {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.in_flow.push(btn_resp.rect);

                    if btn_resp.clicked() && commons.arrow_active {
                        commons.clicked_place_arrow_info = Some((
                            Some(resource.clone()),
                            self.id,
                            resource_flow_index,
                            RecipeWindowType::Basic,
                        ));
                    }
                }
                Io::Output => {}
            }

            text_edit(ui, &mut name);
            ui.label(":");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut amount).ui(ui);
                    ui.label("per cycle");
                });
                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut amount_per_time).ui(ui);
                    ui.label(
                        egui::RichText::new(match rate {
                            RatePer::Tick => "/tick",
                            RatePer::Second => "/s",
                            RatePer::Minute => "/min ",
                            RatePer::Hour => "/h",
                        })
                        .color(color),
                    )
                });
            });

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
                            RecipeWindowType::Basic,
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
            text_edit(ui, &mut name);
            ui.label(":");
            egui::DragValue::new(&mut amount).ui(ui);
            ui.label("per cycle");
        });
    }

    fn show_time_settings(&mut self, ui: &mut egui::Ui, _enabled: bool) -> Result<(), FlowError> {
        let mut amount = self.time_cycle;
        let mut rate = self.time_unit;
        ui.horizontal(|ui| {
            ui.label("Cycle duration:");
            egui::DragValue::new(&mut amount).ui(ui);
            rate_combo(ui, &mut rate);
        });
        let mut changed = false;
        if amount != self.time_cycle {
            self.time_cycle = amount;
            changed = true;
        }

        if rate != self.time_unit {
            self.time_unit = rate;
            changed = true;
        }

        if changed {
            let update_flow = |flow: &mut Vec<ManageFlow<usize>>| -> Result<(), FlowError> {
                for f in flow.iter_mut() {
                    match f {
                        RecipeInput(f) => {
                            f.needed
                                .convert_time_base(self.time_cycle, self.time_unit)?;
                        }
                        RecipeOutput(f) => {
                            f.created
                                .convert_time_base(self.time_cycle, self.time_unit)?;
                        }
                    }
                }
                Ok(())
            };
            update_flow(&mut self.inputs)?;
            update_flow(&mut self.outputs)?;
        }
        Ok(())
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResourceSource {
    ///unique id of the resource
    pub(crate) id: egui::Id,

    ///output
    pub(crate) output: RecipeOutputResource<usize>,

    ///limited output
    pub(crate) limited_output: bool,

    ///limit amount
    limit_amount: usize,

    ///limit rate
    limit_rate: RatePer,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,
}

impl RecipeWindowGUI for ResourceSource {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();

        let mut open = true;

        let resource = self.output.resource();

        let mut resource_name = resource.name.clone();

        let response = egui::Window::new("Resource source")
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    text_edit(ui, &mut resource_name);

                    ui.checkbox(&mut self.limited_output, "Limited");

                    if self.limited_output {
                        egui::DragValue::new(&mut self.limit_amount).ui(ui);
                        rate_combo(ui, &mut self.limit_rate);
                    }

                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.out_flow.push(btn_resp.rect);

                    if btn_resp.clicked() {
                        commons.clicked_start_arrow_info = Some((
                            resource.clone(),
                            self.id,
                            ui.layer_id(),
                            0,
                            RecipeWindowType::Source,
                        ));
                    }
                });
            });
        let inner_response = response.unwrap();
        self.window_coordinate.window = inner_response.response.rect;

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

impl ResourceSource {
    pub fn new(resource: String) -> Self {
        let r = ResourceDefinition {
            name: resource.clone(),
            unit: Unit::Piece,
        };
        Self {
            id: Self::gen_id(resource),
            output: RecipeOutputResource::new(
                r.clone(),
                ResourceFlow::new(&r, 10, 1.0f32, RatePer::Tick),
            ),
            limited_output: false,
            limit_amount: 1,
            limit_rate: RatePer::Second,
            window_coordinate: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct ResourceSink {
    ///unique id of the resource
    pub(crate) id: egui::Id,

    ///output
    pub(crate) sink: Option<RecipeInputResource<usize>>,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,
}

impl ResourceSink {
    pub(crate) fn new() -> Self {
        Self {
            id: Self::gen_id("ResourceSink".to_string()),
            sink: None,
            window_coordinate: Default::default(),
        }
    }
}

impl RecipeWindowGUI for ResourceSink {
    fn show(&mut self, commons: &mut CommonManager, ctx: &egui::Context, enabled: bool) -> bool {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();

        let mut open = true;

        let response = egui::Window::new("Resource sink")
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let btn_resp = ui.button("⭕");

                    self.window_coordinate.in_flow.push(btn_resp.rect);

                    if btn_resp.clicked() && commons.arrow_active {
                        commons.clicked_place_arrow_info = Some((
                            match &self.sink {
                                None => None,
                                Some(sink) => Some(sink.resource().clone()),
                            },
                            self.id,
                            0,
                            RecipeWindowType::Sink,
                        ));
                    }

                    if let Some(sink) = &self.sink {
                        let mut resource_name = sink.resource().name;
                        text_edit(ui, &mut resource_name);
                        let mut amount_per_time = sink.total_in().amount;
                        let mut rate = sink.total_in().rate;
                        egui::DragValue::new(&mut amount_per_time).ui(ui);
                        ui.label(egui::RichText::new(match rate {
                            RatePer::Tick => "/tick",
                            RatePer::Second => "/s",
                            RatePer::Minute => "/min ",
                            RatePer::Hour => "/h",
                        }));
                    }
                });
            });
        let inner_response = response.unwrap();
        self.window_coordinate.window = inner_response.response.rect;

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
        let flow = ResourceFlow::new(&resource, self.amount_per_cycle, 1.0f32, RatePer::Second);
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
    pub(crate) start_flow_type: RecipeWindowType,
    pub(crate) end_flow_window: Option<egui::Id>,
    pub(crate) end_flow_type: Option<RecipeWindowType>,
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
            true => egui::Color32::GRAY,
            false => egui::Color32::BLACK,
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
        start_flow_type: RecipeWindowType,
        layer_id: egui::LayerId,
        flow_index: usize,
    ) -> Self {
        ArrowFlow {
            id: Self::gen_id(format!("Flow{start_flow:?}")),
            state: ArrowUsageState::Active,
            resource,
            start_flow_window: start_flow,
            start_flow_type,
            end_flow_window: None,
            end_flow_type: None,
            start_flow_index: flow_index,
            end_flow_index: 0,
            layer_id,
        }
    }

    pub(crate) fn put_end(
        &mut self,
        resource: Option<ResourceDefinition>,
        end_flow: egui::Id,
        end_flow_type: RecipeWindowType,
        flow_index: usize,
    ) -> Result<(), FlowError> {
        if let Some(resource) = resource {
            if resource != self.resource {
                return Err(FlowError::new(FlowErrorType::WrongResourceType));
            }
        }

        self.end_flow_window = Some(end_flow);
        self.end_flow_type = Some(end_flow_type);
        self.end_flow_index = flow_index;
        self.state = ArrowUsageState::Anchored;

        Ok(())
    }
}
