use num_traits::{Num, One};
use std::ops::AddAssign;

///A trait for any number needed
pub trait Number:
    Num + PartialOrd + Copy + AddAssign + From<usize> + One + eframe::emath::Numeric
{
}
impl<T> Number for T where
    T: Num + PartialOrd + Copy + AddAssign + From<usize> + One + eframe::emath::Numeric
{
}

///A multi purpose enum to differentiate input from outputs
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Io {
    Input,
    Output,
}
