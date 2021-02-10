use crate::validators::type_index::TypeIndexValidationError::InvalidTypeIndex;
use crate::types::TypeIndex;

#[derive(PartialEq, Eq, Debug)]
pub enum TypeIndexValidationError {
    InvalidTypeIndex,
}

pub fn validate_type_index(type_index: &TypeIndex, max_type_index: Option<TypeIndex>) -> Result<(), TypeIndexValidationError> {
    if let Some(index) = max_type_index {
        if *type_index > index {
            return Err(InvalidTypeIndex);
        }
    } else {
        return Err(InvalidTypeIndex);
    }

    Ok(())
}