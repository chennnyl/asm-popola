pub mod instructions;
pub mod vm;
mod parser;
mod util;
pub mod stdlib;

pub mod utility {
    use super::util;
    pub use util::{break_u16, build_u16};
}
