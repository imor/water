use crate::types::{ImportDescriptor, TypeIndex, TableType, Limits, MemoryType};
use crate::types::ImportDescriptor::{Func, Table, Memory};
use crate::validators::import::ImportValidationError::{InvalidFuncTypeIndex, InvalidTableLimits, InvalidMemoryLimits};
use crate::validators::type_index::{TypeIndexValidationError, validate_type_index};

#[derive(PartialEq, Eq, Debug)]
pub enum ImportValidationError {
    InvalidFuncTypeIndex,
    InvalidTableLimits,
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
        Table(TableType { limits }) => {
            //TODO:Why does the spec say that table limits must be valid within 2^32 when the
            //min and max in a limits type are u32? Wouldn't this be always true?
            //see: https://webassembly.github.io/spec/core/valid/types.html#table-types
            if !limits_in_range(&limits, u32::max_value()) {
                return Err(InvalidTableLimits);
            }
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

fn limits_in_range(limits: &Limits, range: u32) -> bool {
    let min = limits.min;
    min <= range && if let Some(max) = limits.max {
        max <= range && min <= max
    } else {
        true
    }
}