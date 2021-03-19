use crate::types::{FuncIndex, TypeIndex, FunctionType};
use crate::validators::start::StartValidationError::{InvalidFuncIndex, InvalidTypeIndex, InvalidType};

#[derive(PartialEq, Eq, Debug)]
pub enum StartValidationError {
    InvalidFuncIndex(FuncIndex),
    InvalidTypeIndex(TypeIndex),
    InvalidType,
}

pub fn validate_start(
    start_func_index: FuncIndex,
    func_type_indices: &[TypeIndex],
    function_types: &[FunctionType]
) -> Result<(), StartValidationError> {
    if start_func_index.0 as usize >= func_type_indices.len() {
        return Err(InvalidFuncIndex(start_func_index));
    }
    let type_index = func_type_indices[start_func_index.0 as usize];

    if type_index.0 as usize >= function_types.len() {
        return Err(InvalidTypeIndex(type_index));
    }

    let func_type = &function_types[type_index.0 as usize];
    if !func_type.params.is_empty() || !func_type.results.is_empty() {
        return Err(InvalidType);
    }
    Ok(())
}