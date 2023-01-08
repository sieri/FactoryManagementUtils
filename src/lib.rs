#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub(crate) mod utils;

pub use app::FactoryManagementUtilsApp;

#[cfg(test)]
mod tests;
