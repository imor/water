use crate::{Chunk, SectionReader, ImportReaderError};
use std::result;
use crate::validators::preamble::{validate_preamble, PreambleValidationError};
use std::cmp::max;
use crate::validators::import::{validate_import_desc, ImportValidationError};

pub struct Validator {
    max_func_index: Option<u32>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ValidationError {
    PreambleValidation(PreambleValidationError),
    ImportValidation(ImportValidationError),
    ImportReader(ImportReaderError),
}

impl From<PreambleValidationError> for ValidationError {
    fn from(e: PreambleValidationError) -> Self {
        ValidationError::PreambleValidation(e)
    }
}

impl From<ImportValidationError> for ValidationError {
    fn from(e: ImportValidationError) -> Self {
        ValidationError::ImportValidation(e)
    }
}

impl From<ImportReaderError> for ValidationError {
    fn from(e: ImportReaderError) -> Self {
        ValidationError::ImportReader(e)
    }
}

pub type Result<T, E = ValidationError> = result::Result<T, E>;

impl Validator {
    pub fn new() -> Validator {
        Validator { max_func_index: None }
    }

    pub fn validate(&mut self, chunk: &Chunk) -> Result<()> {
        match *chunk {
            Chunk::Preamble(magic_number, version) => {
                validate_preamble(magic_number, version)?;
            }
            Chunk::Section(ref section_reader) => {
                match section_reader {
                    SectionReader::Type(reader) => {
                        for (index, _func_type) in reader.clone().into_iter().enumerate() {
                            let current_max = self.max_func_index.unwrap_or(0);
                            self.max_func_index = Some(max(current_max, index as u32));
                        }
                    }
                    SectionReader::Import(reader) => {
                        for import in reader.clone().into_iter() {
                            let import = import?;
                            let id = import.import_descriptor;
                            validate_import_desc(id, self.max_func_index)?
                        }
                    },
                    _ => {}
                }
            }
            Chunk::Done => {
            }
        }
        Ok(())
    }
}