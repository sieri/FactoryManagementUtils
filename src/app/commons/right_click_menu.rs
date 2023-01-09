//TODO: placeholder code for right click investigate

#[allow(dead_code)]
use egui::{Context, Ui};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RightClick {
    pos: egui::Pos2,
}

impl RightClick {
    pub fn new(pos: egui::Pos2) -> Self {
        Self { pos }
    }

    pub fn show<F>(&self, ctx: &Context, f: F)
    where
        F: FnOnce(&mut Ui),
    {
        let r = egui::show_tooltip_at(
            ctx,
            egui::Id::new(format!("{:?}", self.pos)),
            Some(self.pos),
            |ui| {
                (f)(ui);
            },
        );
    }
}
