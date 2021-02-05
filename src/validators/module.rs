use crate::Chunk;
use std::result;
use crate::validators::preamble::{validate_preamble, PreambleValidationError};

pub struct Validator {}

#[derive(PartialEq, Eq, Debug)]
pub enum ValidationError {
    Preamble(PreambleValidationError)
}

impl From<PreambleValidationError> for ValidationError {
    fn from(e: PreambleValidationError) -> Self {
        ValidationError::Preamble(e)
    }
}

pub type Result<T, E = ValidationError> = result::Result<T, E>;

impl Validator {
    pub fn new() -> Validator {
        Validator {}
    }

    pub fn validate(&self, chunk: &Chunk) -> Result<()> {
        match *chunk {
            Chunk::Preamble(magic_number, version) => {
                validate_preamble(magic_number, version)?
            }
            Chunk::Section(_) => {}
            Chunk::Done => {}
        }
        Ok(())
    }
}