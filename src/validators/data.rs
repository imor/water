use crate::types::{MemoryIndex, DataSegment, GlobalType, ValueType};
use crate::validators::code::{CodeValidationError, is_expr_const_and_of_right_type};
use crate::validators::data::DataValidationError::InvalidMemoryIndex;

#[derive(PartialEq, Eq, Debug)]
pub enum DataValidationError {
    InvalidMemoryIndex(MemoryIndex),
    CodeValidation(CodeValidationError),
}

impl From<CodeValidationError> for DataValidationError {
    fn from(e: CodeValidationError) -> Self {
        DataValidationError::CodeValidation(e)
    }
}

pub fn validate_data(
    data_segment: &mut DataSegment,
    max_memory_index: Option<MemoryIndex>,
    globals: &[GlobalType]
) -> Result<(), DataValidationError> {
    if max_memory_index.is_none() || data_segment.memory_index > max_memory_index.unwrap() {
        return Err(InvalidMemoryIndex(data_segment.memory_index));
    }
    is_expr_const_and_of_right_type(
        &mut data_segment.instruction_reader,
        ValueType::I32,
        globals
    )?;
    Ok(())
}
