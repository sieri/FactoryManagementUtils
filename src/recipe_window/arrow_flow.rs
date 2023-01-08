use crate::app::CommonManager;
use crate::recipe_window::{RecipeWindowGUI, RecipeWindowType};
use crate::resources::{FlowError, FlowErrorType, ResourceDefinition};
use std::fmt::Write;
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

    fn generate_tooltip(&self) -> Result<String, std::fmt::Error> {
        Ok("".to_string())
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

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone)]
pub(crate) enum ArrowUsageState {
    Active,
    Anchored,
}
