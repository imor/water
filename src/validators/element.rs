use crate::types::{ElementSegment, TableIndex, ValueType, GlobalType, FuncIndex};
use crate::validators::element::ElementValidationError::{InvalidTableIndex, InvalidFuncIndex};
use crate::validators::code::{is_expr_const_and_of_right_type, CodeValidationError};

#[derive(PartialEq, Eq, Debug)]
pub enum ElementValidationError {
    InvalidTableIndex(TableIndex),
    InvalidFuncIndex(FuncIndex),
    CodeValidation(CodeValidationError),
}

impl From<CodeValidationError> for ElementValidationError {
    fn from(e: CodeValidationError) -> Self {
        ElementValidationError::CodeValidation(e)
    }
}

pub fn validate_element(
    element_segment: &mut ElementSegment,
    max_table_index: Option<TableIndex>,
    max_func_index: Option<FuncIndex>,
    globals: &[GlobalType]
) -> Result<(), ElementValidationError> {
    if max_table_index.is_none() || element_segment.table_index > max_table_index.unwrap() {
        return Err(InvalidTableIndex(element_segment.table_index));
    }
    is_expr_const_and_of_right_type(
        &mut element_segment.instruction_reader,
        ValueType::I32,
        globals
    )?;
    for func_index in &*element_segment.function_indices {
        if max_func_index.is_none() || *func_index > max_func_index.unwrap() {
            return Err(InvalidFuncIndex(*func_index));
        }
    }
    Ok(())
}