pub mod formatting;
pub mod log;

use num_traits::{Float, Num, NumCast, One, ToPrimitive};
use std::fmt::Display;
use std::ops::AddAssign;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Once;

use uuid::{uuid, Uuid};

static ID_PREFIX_1: AtomicU64 = AtomicU64::new(0);
static ID_PREFIX_2: AtomicU64 = AtomicU64::new(0);
static ID_SUFFIX: AtomicU64 = AtomicU64::new(0);

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

pub fn id_init() {
    static ID_INIT: Once = Once::new();
    ID_INIT.call_once(|| {
        let uuid = Uuid::new_v4().as_u64_pair();
        ID_PREFIX_1.store(uuid.0, Ordering::SeqCst);
        ID_PREFIX_2.store(uuid.1, Ordering::SeqCst);
    });
}

pub fn gen_id(name: String) -> egui::Id {
    let id_suffix = ID_SUFFIX.fetch_add(1, Ordering::SeqCst);
    let id_prefix_1 = ID_PREFIX_1.load(Ordering::SeqCst);
    let id_prefix_2 = ID_PREFIX_2.load(Ordering::SeqCst);
    egui::Id::new(&*format!("{}{}{}", id_prefix_1, id_prefix_2, id_suffix))
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
