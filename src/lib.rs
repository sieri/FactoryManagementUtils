#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod recipe_window;
pub(crate) mod resources;
pub(crate) mod utils;

pub use app::FactoryManagementUtilsApp;

#[cfg(test)]
mod tests;
