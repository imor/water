use crate::types::Limits;

pub mod module;
pub mod preamble;
mod import;
mod type_index;

fn limits_in_range(limits: &Limits, range: u32) -> bool {
    let min = limits.min;
    min <= range && if let Some(max) = limits.max {
        max <= range && min <= max
    } else {
        true
    }
}
