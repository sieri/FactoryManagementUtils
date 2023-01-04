use num_traits::{Float, Num, NumCast, One, ToPrimitive};
use std::fmt::Display;
use std::ops::AddAssign;

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
pub trait FloatingNumber: Number + Float + From<f32> {}
impl<T> FloatingNumber for T where T: Number + Float + From<f32> + NumCast {}

///A multi purpose enum to differentiate input from outputs
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Io {
    Input,
    Output,
}
