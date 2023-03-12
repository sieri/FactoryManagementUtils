pub mod formatting;
pub mod log;

use num_traits::{Float, Num, NumCast, One, ToPrimitive};
use std::fmt::Display;
use std::ops::AddAssign;
use std::time::{SystemTime, UNIX_EPOCH};

///A trait for any number needed
pub trait Number:
    Num + PartialOrd + Copy + AddAssign + One + eframe::emath::Numeric + ToPrimitive + Display
{
}
impl<T> Number for T where
    T: Num
        + PartialOrd
        + Copy
        + AddAssign
        + One
        + eframe::emath::Numeric
        + NumCast
        + ToPrimitive
        + Display
{
}

///A trait for any float needed
pub trait FloatingNumber: Number + Float + From<f32> + PartialOrd + ToPrimitive {}
impl<T> FloatingNumber for T where T: Number + Float + From<f32> + NumCast + PartialOrd + ToPrimitive
{}

///A multi purpose enum to differentiate input from outputs
#[derive(serde::Deserialize, serde::Serialize, Debug, Eq, PartialEq, Clone)]
pub enum Io {
    Input,
    Output,
}

pub fn gen_id(name: String) -> egui::Id {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    egui::Id::new(name + &*format!("{timestamp}"))
}

#[cfg(test)]
pub mod test_env {

    use crate::utils::log::setup_logger;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            setup_logger().expect("Logger couldn't be initialized");
        });
    }
}
