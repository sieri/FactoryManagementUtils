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
    fn show(&self, ctx: &egui::Context, enabled: bool) -> bool;
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Descriptor for a Basic Recipe window, the recipe is directly calculated
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
    fn show(&self, ctx: &egui::Context, enabled: bool) -> bool {
        let mut open = true;
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .enabled(enabled)
            .open(&mut open)
            .show(ctx, |ui| ui.label("Default recipe"));

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
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = egui::Id::new(title.to_owned() + &*format!("{}", timestamp));
        Self { title, id }
    }
}
