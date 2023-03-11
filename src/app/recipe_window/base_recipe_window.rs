use crate::app::commons::CommonsManager;
use crate::app::coordinates_info::CoordinatesInfo;
use crate::app::error::ShowError;
use crate::app::recipe_window;
use crate::app::recipe_window::resource_adding_window::ResourceAddingWindow;
use crate::app::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::app::resources::recipe_output_resource::RecipeOutputResource;
use crate::app::resources::resource_flow::{ManageResourceFlow, ResourceFlow};
use crate::app::resources::ManageFlow::{RecipeInput, RecipeOutput};
use crate::app::resources::{FlowError, ManageFlow, RatePer, ResourceDefinition, Unit};
use crate::utils::{gen_id, Io};
use egui::{Context, InnerResponse, Widget};
use itertools::{EitherOrBoth, Itertools};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Write};
use std::io::Cursor;
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(default)]
/// Descriptor for a Base Recipe window, the recipe is directly calculated
pub struct BaseRecipeWindow {
    ///Title of the recipe
    pub(crate) title: String,

    ///unique id of the recipe
    pub(crate) id: egui::Id,

    ///unique id for the tooltip
    pub(crate) tooltip_id: egui::Id,

    ///unique id for the temporary tooltip
    temp_tooltip_id: egui::Id,

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

    ///Description
    description: String,

    ///flag if the description open
    description_open: bool,

    ///Flag indicating if every inputs have sufficient resources
    stable_in: bool,

    ///Flag indicating if every outputs have sufficient draining
    stable_out: bool,

    ///Configurations of the features shown
    config: ConfigFeatures,

    ///the type of the recipe
    recipe_type: RecipeWindowType,

    #[serde(skip)]
    window_coordinate: CoordinatesInfo,

    #[serde(skip)]
    pub(crate) errors: Vec<ShowError>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub struct ConfigFeatures {
    pub interactive_input: bool,
    pub pure_time_input: bool,
    pub interactive_output: bool,
    pub pure_time_output: bool,
    pub show_power: bool,
    pub show_time: bool,
}

impl Default for BaseRecipeWindow {
    fn default() -> Self {
        Self::new(
            String::from("Basic Recipe Window"),
            ConfigFeatures {
                interactive_input: true,
                pure_time_input: true,
                interactive_output: true,
                pure_time_output: true,
                show_power: true,
                show_time: true,
            },
            RecipeWindowType::SimpleRecipe,
        )
    }
}

impl BaseRecipeWindow {
    /// Create a new basic recipe
    ///
    /// # Arguments
    ///
    /// * `title`: Title of the recipe
    ///
    /// returns: BasicRecipeWindowDescriptor
    pub fn new(title: String, config: ConfigFeatures, recipe_type: RecipeWindowType) -> Self {
        Self::new_with_custom_output(
            config,
            recipe_type,
            ResourceFlow::new(
                &ResourceDefinition {
                    name: title.to_string(),
                    unit: Unit::Piece,
                },
                1,
                1.0,
                RatePer::Second,
            ),
        )
    }

    pub fn new_with_custom_output(
        config: ConfigFeatures,
        recipe_type: RecipeWindowType,
        flow: ResourceFlow<usize, f32>,
    ) -> Self {
        let title = flow.resource.name.clone();
        let id = gen_id(title.clone());
        let tooltip_id = id.with("Tooltip");
        let temp_tooltip_id = id.with("Temp Tooltip");
        let resource = flow.resource.clone();

        let output = RecipeOutput(RecipeOutputResource::new(resource, flow));
        Self {
            title,
            id,
            tooltip_id,
            temp_tooltip_id,
            inputs: vec![],
            outputs: vec![output],
            power: None,
            resource_adding_windows: vec![],
            time_cycle: 1,
            time_unit: RatePer::Second,
            description: "".to_string(),
            description_open: false,
            stable_in: false,
            stable_out: false,
            config,
            recipe_type,
            window_coordinate: CoordinatesInfo::default(),
            errors: vec![],
        }
    }

    pub(crate) fn show_inputs(
        &mut self,
        commons: &mut CommonsManager,
        ui: &mut egui::Ui,
        enabled: bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Inputs");
                if self.config.interactive_input
                    && ui
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
                self.show_flow(
                    commons,
                    i,
                    Io::Input,
                    ui,
                    enabled,
                    self.config.pure_time_input,
                );
            }
        });
    }

    pub(crate) fn show_outputs(
        &mut self,
        commons: &mut CommonsManager,
        ui: &mut egui::Ui,
        enabled: bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let _ = ui.label("Outputs");
                if self.config.interactive_input
                    && ui
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
                self.show_flow(
                    commons,
                    i,
                    Io::Output,
                    ui,
                    enabled,
                    self.config.pure_time_output,
                );
            }
        });
    }

    fn show_flow(
        &mut self,
        commons: &mut CommonsManager,
        resource_flow_index: usize,
        dir: Io,
        ui: &mut egui::Ui,
        _enabled: bool,
        pure_time: bool,
    ) {
        let mut changed = false;
        //get variables
        let resource_flow: &mut dyn ManageResourceFlow<usize> = match dir {
            Io::Input => match &mut self.inputs[resource_flow_index] {
                RecipeInput(r) => r,
                RecipeOutput(_) => {
                    panic!("Error!!! Impossible situation")
                }
            },
            Io::Output => match &mut self.outputs[resource_flow_index] {
                RecipeInput(_) => {
                    panic!("Error!!! Impossible situation")
                }
                RecipeOutput(r) => r,
            },
        };
        let resource = resource_flow.resource();

        let mut name = resource.clone().name;

        self.stable_in &= resource_flow.is_enough();

        let color = match resource_flow.is_enough() {
            true => egui::Rgba::GREEN,
            false => egui::Rgba::RED,
        };

        let flow = match dir {
            Io::Input => resource_flow.total_out(),
            Io::Output => resource_flow.total_in(),
        };
        let rate = flow.rate;
        let mut amount = flow.amount_per_cycle;
        let mut amount_per_time = flow.amount;

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
                            self.recipe_type,
                        ));
                    }
                }
                Io::Output => {}
            }

            recipe_window::text_edit(ui, &mut name);
            ui.label(":");

            ui.vertical(|ui| {
                if !pure_time {
                    ui.horizontal(|ui| {
                        changed |= egui::DragValue::new(&mut amount).ui(ui).changed();
                        ui.label("per cycle");
                    });
                }
                ui.horizontal(|ui| {
                    changed |= egui::DragValue::new(&mut amount_per_time).ui(ui).changed();
                    ui.label(egui::RichText::new(rate.to_shortened_string()).color(color))
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
                            self.recipe_type,
                        ));
                    }
                }
            }
        });

        if changed {
            if amount != flow.amount_per_cycle {
                match dir {
                    Io::Input => {
                        if self.config.interactive_input {
                            resource_flow.set_designed_amount_per_cycle(amount);
                        }
                    }
                    Io::Output => {
                        if self.config.interactive_output {
                            resource_flow.set_designed_amount_per_cycle(amount);
                        }
                    }
                }
            }

            commons.recalculate = true;
            let r = self.update_flow(dir);
            if let Err(e) = r {
                commons.add_error(ShowError::new(e.to_string()));
            }
        }

        commons.recalculate |= changed;
    }

    pub(crate) fn show_power(&mut self, ui: &mut egui::Ui, _enabled: bool) {
        let power: &dyn ManageResourceFlow<usize> = match &self.power {
            None => {
                if ui.button("Add Power").clicked() {} //TODO: Add power
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
            recipe_window::text_edit(ui, &mut name);
            ui.label(":");
            egui::DragValue::new(&mut amount).ui(ui);
            ui.label("per cycle");
        });
    }

    pub(crate) fn show_time_settings(
        &mut self,
        mut common: &mut CommonsManager,
        ui: &mut egui::Ui,
        _enabled: bool,
    ) -> Result<(), FlowError> {
        let mut amount = self.time_cycle;
        let mut rate = self.time_unit;
        ui.horizontal(|ui| {
            ui.label("Cycle duration:");
            egui::DragValue::new(&mut amount).ui(ui);
            recipe_window::rate_combo(ui, &mut rate);
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
            common.recalculate = true;
            self.update_flow(Io::Input)?;
            self.update_flow(Io::Output)?;
        }
        Ok(())
    }

    pub(crate) fn show_notes(&mut self, ui: &mut egui::Ui, _enabled: bool) {
        let short_title = self.description.lines().next().unwrap_or("").trim();
        egui::CollapsingHeader::new(format!("Notes: {short_title}"))
            .id_source(self.id)
            .show(ui, |ui| {
                ui.text_edit_multiline(&mut self.description);
            });
    }

    pub(crate) fn update_flow(&mut self, dir: Io) -> Result<(), FlowError> {
        let flow = match dir {
            Io::Input => &mut self.inputs,
            Io::Output => &mut self.outputs,
        };
        let mut stable = true;
        for f in flow.iter_mut() {
            match f {
                RecipeInput(f) => {
                    stable &= f.is_enough();
                    if !self.config.pure_time_input {
                        f.needed
                            .convert_time_base(self.time_cycle, self.time_unit)?;
                    }
                }
                RecipeOutput(f) => {
                    stable &= f.is_enough();
                    if !self.config.pure_time_output {
                        f.created
                            .convert_time_base(self.time_cycle, self.time_unit)?;
                    }
                }
            }
        }

        match dir {
            Io::Input => self.stable_in = stable,
            Io::Output => self.stable_out = stable,
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

    pub(crate) fn get_title(&self) -> String {
        self.title.clone()
    }
    pub(crate) fn gen_ids(&mut self) {
        self.id = gen_id(self.title.clone());
        self.tooltip_id = self.id.with("Tooltip");
        self.temp_tooltip_id = self.id.with("Temp Tooltip")
    }

    pub(crate) fn clean_coordinates(&mut self) {
        self.window_coordinate.in_flow.clear();
        self.window_coordinate.out_flow.clear();
    }

    pub(crate) fn gen_title_string(&mut self) -> String {
        format!(
            "{}{}{}",
            self.title,
            match self.stable_in {
                true => {
                    "✔"
                }
                false => {
                    "❗"
                }
            },
            match self.stable_out {
                true => {
                    ""
                }
                false => {
                    "⛔"
                }
            }
        )
    }

    pub(crate) fn window(
        &mut self,
        commons: &mut CommonsManager,
        ctx: &Context,
        enabled: bool,
        open: &mut bool,
        title: String,
    ) -> Option<InnerResponse<Option<()>>> {
        let response = egui::Window::new(title)
            .id(self.id)
            .enabled(enabled)
            .open(open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        self.show_inputs(commons, ui, enabled);
                        ui.separator();
                        self.show_outputs(commons, ui, enabled);
                    });
                    ui.separator();
                    if self.config.show_power {
                        self.show_power(ui, enabled);
                    }
                    if self.config.show_time {
                        let result = self.show_time_settings(commons, ui, enabled);
                        if result.is_err() {
                            commons.add_error(ShowError::new(format!("{}", result.err().unwrap())));
                        }
                    }
                    self.show_notes(ui, enabled);
                });
            });
        response
    }

    pub(crate) fn update_coordinates(&mut self, inner_response: &InnerResponse<Option<()>>) {
        self.window_coordinate.window = inner_response.response.rect;
    }

    pub(crate) fn show_tooltips(
        &mut self,
        commons: &mut CommonsManager,
        ctx: &Context,
        inner_response: InnerResponse<Option<()>>,
    ) {
        let mut resp = inner_response.response;
        if commons.has_tooltip(self.temp_tooltip_id) {
            resp = resp.on_hover_ui(|ui| {
                commons.tooltip(ctx, ui, self.temp_tooltip_id, Duration::from_secs(2));
            });
        }

        if inner_response.inner.is_none() {
            resp.on_hover_ui(|ui| {
                ui.label(
                    egui::RichText::new(
                        self.generate_tooltip()
                            .unwrap_or_else(|_| "Error generating tooltip".to_string()),
                    )
                    .font(egui::FontId::monospace(10.0)),
                );
            });
        }
    }

    pub(crate) fn show_resource_adding_windows(
        &mut self,
        commons: &mut CommonsManager,
        ctx: &Context,
        enabled: bool,
    ) {
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
    }

    pub(crate) fn push_coordinates(&mut self, commons: &mut CommonsManager, open: &mut bool) {
        if *open {
            commons
                .window_coordinates
                .insert(self.id, self.window_coordinate.clone());
        } else {
            commons.window_coordinates.remove(&self.id);
        }
    }

    pub(crate) fn push_errors(&mut self, commons: &mut CommonsManager) {
        if let Some(err) = self.errors.pop() {
            commons.show_errors.push_back(err);
        }
    }

    pub(crate) fn generate_tooltip(&self) -> Result<String, std::fmt::Error> {
        //TODO: FIX
        let mut tooltip = String::new();

        let mut colum_a = Vec::new();
        let mut colum_b = Vec::new();
        let mut colum_a_lengths = (0, 0, 0);
        let mut colum_b_lengths = (0, 0, 0);
        writeln!(tooltip, "{}", self.title)?;
        let input_title = "Inputs:".to_string();
        let output_title = "Outputs:".to_string();
        colum_a_lengths.0 = input_title.len();

        colum_b_lengths.0 = output_title.len();

        for it in self.inputs.iter().zip_longest(self.outputs.iter()) {
            match it {
                EitherOrBoth::Both(input, output) => {
                    let temp = input.to_split_string();
                    colum_a_lengths = (
                        colum_a_lengths.0.max(temp[0].len()),
                        colum_a_lengths.1.max(temp[1].len()),
                        colum_a_lengths.2.max(temp[2].len()),
                    );
                    colum_a.push((
                        Some(temp[0].clone()),
                        Some(temp[1].clone()),
                        Some(temp[2].clone()),
                    ));
                    let temp = output.to_split_string();
                    colum_b_lengths = (
                        colum_b_lengths.0.max(temp[0].len()),
                        colum_b_lengths.1.max(temp[1].len()),
                        colum_b_lengths.2.max(temp[2].len()),
                    );
                    colum_b.push((
                        Some(temp[0].clone()),
                        Some(temp[1].clone()),
                        Some(temp[2].clone()),
                    ));
                }
                EitherOrBoth::Left(input) => {
                    let temp = input.to_split_string();
                    colum_a_lengths = (
                        colum_a_lengths.0.max(temp[0].len()),
                        colum_a_lengths.1.max(temp[1].len()),
                        colum_a_lengths.2.max(temp[2].len()),
                    );
                    colum_a.push((
                        Some(temp[0].clone()),
                        Some(temp[1].clone()),
                        Some(temp[2].clone()),
                    ));
                }
                EitherOrBoth::Right(output) => {
                    let temp = output.to_split_string();
                    colum_b_lengths = (
                        colum_b_lengths.0.max(temp[0].len()),
                        colum_b_lengths.1.max(temp[1].len()),
                        colum_b_lengths.2.max(temp[2].len()),
                    );
                    colum_b.push((
                        Some(temp[0].clone()),
                        Some(temp[1].clone()),
                        Some(temp[2].clone()),
                    ));
                }
            }
        }

        let a0_len = colum_a_lengths.0;
        let a1_len = colum_a_lengths.1;
        let a2_len = colum_a_lengths.0 + colum_a_lengths.1 + 1;
        let b0_len = colum_b_lengths.0;
        let b1_len = colum_b_lengths.1;
        let b2_len = colum_b_lengths.0 + colum_b_lengths.1 - 1;
        let empty = "".to_string();

        debug!(
            "a0_len = {}\na1_len = {}\na2_len = {}\nb0_len = {}\nb1_len = {}\nb2_len = {}\n",
            a0_len, a1_len, a2_len, b0_len, b1_len, b2_len
        );

        writeln!(
            tooltip,
            "{:<in_len$}|{:<out_len$} ",
            "Inputs: ",
            "Outputs: ",
            in_len = colum_a_lengths.0 + colum_a_lengths.1 - 1,
            out_len = colum_b_lengths.0 + colum_b_lengths.1 + 1
        )?;
        for i in colum_a.iter().zip_longest(colum_b.iter()) {
            match i {
                EitherOrBoth::Both(a, b) => {
                    write!(
                        tooltip,
                        "{a0:<a0_len$}: {a1:>a1_len$}|{b0:<b0_len$}: {b1:>b1_len$}\n{a2:>a2_len$}|   {b2:>b2_len$}\n",
                        a0=a.0.as_ref().unwrap_or(&empty),
                        a1=a.1.as_ref().unwrap_or(&empty),
                        a2=a.2.as_ref().unwrap_or(&empty),
                        b0=b.0.as_ref().unwrap_or(&empty),
                        b1=b.1.as_ref().unwrap_or(&empty),
                        b2=b.2.as_ref().unwrap_or(&empty),
                        a0_len = a0_len,
                        a1_len = a1_len,
                        a2_len = a2_len,
                        b0_len = b0_len,
                        b1_len = b1_len,
                        b2_len = b2_len
                    )?;
                }
                EitherOrBoth::Left(a) => {
                    write!(
                        tooltip,
                        "{a0:<a0_len$}: {a1:>a1_len$}|{b0:<b0_len$}  {b1:>b1_len$}\n{a2:>a2_len$}|   {b2:>b2_len$}\n",
                        a0=a.0.as_ref().unwrap_or(&empty),
                        a1=a.1.as_ref().unwrap_or(&empty),
                        a2=a.2.as_ref().unwrap_or(&empty),
                        b0=empty,
                        b1=empty,

                        b2=empty,
                        a0_len = a0_len,
                        a1_len = a1_len,
                        a2_len = a2_len,
                        b0_len = b0_len,
                        b1_len = b1_len,
                        b2_len = b2_len
                    )?;
                }
                EitherOrBoth::Right(b) => {
                    write!(
                        tooltip,
                        "{a0:<a0_len$} {a1:>a1_len$}|{b0:<b0_len$}: {b1:>b1_len$}\n{a2:>a2_len$}|    {b2:>b2_len$}\n",
                        a0=empty,
                        a1=empty,
                        a2=empty,
                        b0=b.0.as_ref().unwrap_or(&empty),
                        b1=b.1.as_ref().unwrap_or(&empty),

                        b2=b.2.as_ref().unwrap_or(&empty),
                        a0_len = a0_len,
                        a1_len = a1_len,
                        a2_len = a2_len,
                        b0_len = b0_len,
                        b1_len = b1_len,
                        b2_len = b2_len
                    )?;
                }
            }
        }
        Ok(tooltip)
    }
}

impl PartialEq<Self> for BaseRecipeWindow {
    fn eq(&self, other: &Self) -> bool {
        let mut r = true;
        r &= self.id == other.id;
        r &= self.tooltip_id == other.tooltip_id;
        r &= self.temp_tooltip_id == other.temp_tooltip_id;
        r &= self.equivalent(other);
        r
    }
}

impl BaseRecipeWindow {
    pub fn equivalent(&self, other: &BaseRecipeWindow) -> bool {
        let mut r = true;
        r &= self.title == other.title;
        r &= self.inputs == other.inputs;
        r &= self.outputs == other.outputs;
        r &= self.power == other.power;
        r &= self.resource_adding_windows == other.resource_adding_windows;
        r &= self.time_cycle == other.time_cycle;
        r &= self.time_unit == other.time_unit;
        r &= self.description == other.description;
        r &= self.description_open == other.description_open;
        r &= self.stable_in == other.stable_in;
        r &= self.stable_out == other.stable_out;
        r
    }
}

impl Eq for BaseRecipeWindow {}

pub trait RecipeWindowUser<'a>: serde::Serialize {
    type Gen: serde::Serialize + serde::Deserialize<'a> + RecipeWindowUser<'a>;

    fn recipe(self) -> BaseRecipeWindow;

    ///Save to self to json data
    ///None if there was an error, which is added to the error queue
    fn save(&mut self) -> Option<String> {
        let mut vec = vec![0u8];
        let s = Cursor::new(&mut vec);
        let result = self.serialize(&mut serde_json::Serializer::new(s));

        match result {
            Ok(_) => {
                let result = String::from_utf8(vec);
                match result {
                    Ok(s) => Some(s),
                    Err(e) => {
                        self.push_errors(ShowError::new_custom_context(
                            e.to_string(),
                            "An error happened on save of a recipe".to_string(),
                        ));

                        None
                    }
                }
            }
            Err(e) => {
                self.push_errors(ShowError::new_custom_context(
                    e.to_string(),
                    "An error happened on save of a recipe".to_string(),
                ));
                None
            }
        }
    }

    fn push_errors(&mut self, e: ShowError);

    fn load(str: String) -> Result<Self::Gen, ShowError> {
        let cursor = Cursor::new(str);
        let mut des = serde_json::Deserializer::from_reader(cursor);
        let result = Self::Gen::deserialize(&mut des);

        match result {
            Ok(mut loaded) => {
                loaded.gen_ids();
                Ok(loaded)
            }
            Err(e) => Err(ShowError::new_custom_context(
                e.to_string(),
                "An error happened on load of a recipe".to_string(),
            )),
        }
    }

    fn gen_ids(&mut self);

    fn internal_calculation(&mut self);

    fn back_propagation_internal_calculation(
        &mut self,
        rate: f32,
        amount: Option<ResourceFlow<usize, f32>>,
    );
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::resources::resource_flow::ResourceFlow;
    use crate::app::resources::{RatePer, ResourceDefinition};

    #[derive(Debug)]
    pub(crate) struct RecipeResourceInfos {
        pub def: ResourceDefinition,
        pub amount: f32,
        pub amount_per_cycle: usize,
        pub rate: RatePer,
    }
    impl From<&ResourceFlow<usize, f32>> for RecipeResourceInfos {
        fn from(value: &ResourceFlow<usize, f32>) -> Self {
            RecipeResourceInfos {
                def: value.resource.clone(),
                amount: value.amount,
                amount_per_cycle: value.amount_per_cycle,
                rate: value.rate,
            }
        }
    }
}
