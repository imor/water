use crate::types::{GlobalSegment, GlobalType};
use crate::InstructionReaderError;
use crate::validators::code::{is_expr_const_and_of_right_type, CodeValidationError};

#[derive(PartialEq, Eq, Debug)]
pub enum GlobalValidationError {
    CodeValidation(CodeValidationError),
    InstructionReaderError(InstructionReaderError),
}

impl From<InstructionReaderError> for GlobalValidationError {
    fn from(e: InstructionReaderError) -> Self {
        GlobalValidationError::InstructionReaderError(e)
    }
}

impl From<CodeValidationError> for GlobalValidationError {
    fn from(e: CodeValidationError) -> Self {
        GlobalValidationError::CodeValidation(e)
    }
}

pub fn validate_global_type(global: &mut GlobalSegment, globals: &[GlobalType]) -> Result<(), GlobalValidationError>{
    is_expr_const_and_of_right_type(
        &mut global.instruction_reader,
        global.global_type.var_type,
        globals
    )?;
    Ok(())
}