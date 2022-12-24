use std::time::{SystemTime, UNIX_EPOCH};

pub trait RecipeWindowGUI {
    fn spawn(&self, ctx: &egui::Context, enabled: bool);
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BasicRecipeWindowDescriptor {
    title: String,
    id: egui::Id,
}

impl Default for BasicRecipeWindowDescriptor {
    fn default() -> Self {
        Self::new(String::from("Basic Recipe Window"))
    }
}

impl RecipeWindowGUI for BasicRecipeWindowDescriptor {
    fn spawn(&self, ctx: &egui::Context, enabled: bool) {
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .show(ctx, |ui| ui.label("Default recipe"));
    }
}

impl BasicRecipeWindowDescriptor {
    pub fn new(title: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = egui::Id::new(title.to_owned() + &*format!("{}", timestamp));
        Self { title, id }
    }
}
