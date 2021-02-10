use crate::types::{ImportDescriptor, TypeIndex};
use crate::types::ImportDescriptor::{Func, Memory};
use crate::validators::import::ImportValidationError::{InvalidFuncTypeIndex, InvalidMemoryTypeMemoryLimits};
use crate::validators::type_index::{TypeIndexValidationError, validate_type_index};
use crate::validators::memory::{MemoryLimitsValidationError, validate_memory_type};

#[derive(PartialEq, Eq, Debug)]
pub enum ImportValidationError {
    InvalidFuncTypeIndex,
    InvalidMemoryTypeMemoryLimits,
}

impl From<TypeIndexValidationError> for ImportValidationError {
    fn from(_: TypeIndexValidationError) -> Self {
        InvalidFuncTypeIndex
    }
}

impl From<MemoryLimitsValidationError> for ImportValidationError {
    fn from(_: MemoryLimitsValidationError) -> Self {
        InvalidMemoryTypeMemoryLimits
    }
}

pub(crate) fn validate_import_desc(import_desc: ImportDescriptor, max_type_index: Option<TypeIndex>) -> Result<(), ImportValidationError> {
    match import_desc {
        Func { type_index } => {
            validate_type_index(&type_index, max_type_index)?
        },
        Memory(memory) => {
            validate_memory_type(&memory)?
        },
        _ => {},
    }
    Ok(())
}
