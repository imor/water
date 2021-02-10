use crate::types::Limits;
use crate::validators::memory::MemoryLimitsValidationError::InvalidMemoryLimits;

fn limits_in_range(limits: &Limits, range: u32) -> bool {
    let min = limits.min;
    min <= range && if let Some(max) = limits.max {
        max <= range && min <= max
    } else {
        true
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum MemoryLimitsValidationError {
    InvalidMemoryLimits,
}

pub fn validate_memory_limits(limits: &Limits) -> Result<(), MemoryLimitsValidationError> {
    if !limits_in_range(limits, 65536) {
        return Err(InvalidMemoryLimits);
    }
    Ok(())
}
