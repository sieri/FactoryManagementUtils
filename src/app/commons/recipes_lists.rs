use crate::app::error::ShowError;
use crate::app::recipe_window::basic_recipe_window::BasicRecipeWindow;
use eframe::epaint::ahash::HashMapExt;
use egui::epaint::ahash::HashMap;
use egui::Ui;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SavedRecipes {
    content: HashMap<String, String>,
    current: Option<(String, String)>,
}

impl SavedRecipes {
    pub(crate) fn clear(&mut self) {
        self.content.clear();
        self.current = None;
    }
}

impl SavedRecipes {
    pub fn recipes_list(&mut self, ui: &mut Ui) {
        let default = ("Select a recipe:".to_string(), "".to_string());
        egui::ComboBox::from_label("")
            .width(ui.available_width() - 10.0)
            .selected_text(self.current.as_ref().unwrap_or(&default).0.clone())
            .show_ui(ui, |ui| {
                for (title, recipe) in self.content.iter() {
                    ui.selectable_value(
                        &mut self.current,
                        Some((title.clone(), recipe.clone())),
                        title,
                    );
                }
            });
    }

    pub(crate) fn push(&mut self, title: String, data: String) {
        self.content.insert(title, data);
    }

    pub(crate) fn load(&self) -> Result<BasicRecipeWindow, ShowError> {
        match &self.current {
            None => Err(ShowError::new("Please select a recipe".to_string())),
            Some((_, data)) => BasicRecipeWindow::load(data.clone()),
        }
    }
}

impl Default for SavedRecipes {
    fn default() -> Self {
        Self {
            content: HashMap::new(),
            current: None,
        }
    }
}
