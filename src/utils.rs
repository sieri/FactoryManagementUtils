use num_traits::Num;
use std::ops::AddAssign;

///A trait for any number needed
pub trait Number: Num + PartialOrd + Copy + AddAssign + From<usize> {}
