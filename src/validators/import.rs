use crate::types::{ImportDescriptor, TypeIndex, MemoryType};
use crate::types::ImportDescriptor::{Func, Memory};
use crate::validators::import::ImportValidationError::{InvalidFuncTypeIndex, InvalidMemoryLimits};
use crate::validators::type_index::{TypeIndexValidationError, validate_type_index};
use crate::validators::limits_in_range;

#[derive(PartialEq, Eq, Debug)]
pub enum ImportValidationError {
    InvalidFuncTypeIndex,
    InvalidMemoryLimits,
}

impl From<TypeIndexValidationError> for ImportValidationError {
    fn from(_: TypeIndexValidationError) -> Self {
        InvalidFuncTypeIndex
    }
}

pub(crate) fn validate_import_desc(import_desc: ImportDescriptor, max_type_index: Option<TypeIndex>) -> Result<(), ImportValidationError> {
    match import_desc {
        Func { type_index } => {
            validate_type_index(&type_index, max_type_index)?
        },
        Memory(MemoryType { limits }) => {
            if !limits_in_range(&limits, 65536) {
                return Err(InvalidMemoryLimits);
            }
        },
        _ => {},
    }
    Ok(())
}
