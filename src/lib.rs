#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub(crate) mod utils;

pub use app::FactoryManagementApp;

#[cfg(test)]
mod test_framework;
