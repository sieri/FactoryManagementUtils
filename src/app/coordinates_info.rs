#[derive(Clone)]
pub struct CoordinatesInfo {
    pub(crate) window: egui::Rect,
    pub(crate) out_flow: Vec<egui::Rect>,
    pub(crate) in_flow: Vec<egui::Rect>,
}

impl Default for CoordinatesInfo {
    fn default() -> Self {
        CoordinatesInfo {
            window: egui::Rect {
                min: Default::default(),
                max: Default::default(),
            },
            out_flow: vec![],
            in_flow: vec![],
        }
    }
}
