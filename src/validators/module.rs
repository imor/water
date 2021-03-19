use crate::{Chunk, SectionReader, ImportReaderError, FunctionReaderError, TableReaderError, MemoryReaderError, GlobalReaderError, ExportReaderError};
use std::result;
use crate::validators::preamble::{validate_preamble, PreambleValidationError};
use std::cmp::max;
use crate::validators::import::{validate_import_desc, ImportValidationError};
use crate::validators::type_index::{validate_type_index, TypeIndexValidationError};
use crate::types::{TypeIndex, GlobalType, ImportDescriptor, FuncIndex, TableIndex, MemoryIndex, GlobalIndex};
use crate::validators::memory::{validate_memory_type, MemoryLimitsValidationError};
use crate::validators::global::{validate_global_type, GlobalValidationError};
use crate::validators::export::{ExportValidator, ExportValidationError};

pub struct Validator {
    max_type_index: Option<TypeIndex>,
    globals: Vec<GlobalType>,
    max_func_index: Option<FuncIndex>,
    max_table_index: Option<TableIndex>,
    max_memory_index: Option<MemoryIndex>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ValidationError {
    PreambleValidation(PreambleValidationError),
    ImportValidation(ImportValidationError),
    ImportReader(ImportReaderError),
    FunctionValidation(TypeIndexValidationError),
    FunctionReader(FunctionReaderError),
    TableReader(TableReaderError),
    MemoryReader(MemoryReaderError),
    MemoryValidation(MemoryLimitsValidationError),
    GlobalReader(GlobalReaderError),
    GlobalValidation(GlobalValidationError),
    ExportReader(ExportReaderError),
    ExportValidation(ExportValidationError),
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

impl From<TypeIndexValidationError> for ValidationError {
    fn from(e: TypeIndexValidationError) -> Self {
        ValidationError::FunctionValidation(e)
    }
}

impl From<ImportReaderError> for ValidationError {
    fn from(e: ImportReaderError) -> Self {
        ValidationError::ImportReader(e)
    }
}

impl From<FunctionReaderError> for ValidationError {
    fn from(e: FunctionReaderError) -> Self {
        ValidationError::FunctionReader(e)
    }
}

impl From<TableReaderError> for ValidationError {
    fn from(e: TableReaderError) -> Self {
        ValidationError::TableReader(e)
    }
}

impl From<MemoryReaderError> for ValidationError {
    fn from(e: MemoryReaderError) -> Self {
        ValidationError::MemoryReader(e)
    }
}

impl From<MemoryLimitsValidationError> for ValidationError {
    fn from(e: MemoryLimitsValidationError) -> Self {
        ValidationError::MemoryValidation(e)
    }
}

impl From<GlobalReaderError> for ValidationError {
    fn from(e: GlobalReaderError) -> Self {
        ValidationError::GlobalReader(e)
    }
}

impl From<GlobalValidationError> for ValidationError {
    fn from(e: GlobalValidationError) -> Self {
        ValidationError::GlobalValidation(e)
    }
}

impl From<ExportReaderError> for ValidationError {
    fn from(e: ExportReaderError) -> Self {
        ValidationError::ExportReader(e)
    }
}

impl From<ExportValidationError> for ValidationError {
    fn from(e: ExportValidationError) -> Self {
        ValidationError::ExportValidation(e)
    }
}

pub type Result<T, E = ValidationError> = result::Result<T, E>;

impl Validator {
    pub fn new() -> Validator {
        Validator {
            max_type_index: None,
            globals: Vec::new(),
            max_func_index: None,
            max_table_index: None,
            max_memory_index: None,
        }
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
                            let current_max = self.max_type_index.unwrap_or(TypeIndex(0));
                            self.max_type_index = Some(max(current_max, TypeIndex(index as u32)));
                        }
                    }
                    SectionReader::Import(reader) => {
                        for import in reader.clone() {
                            let import = import?;
                            let import_desc = import.import_descriptor;
                            validate_import_desc(&import_desc, self.max_type_index)?;
                            self.add_import_desc(&import_desc);
                        }
                    },
                    SectionReader::Function(reader) => {
                        for type_index in reader.clone() {
                            let type_index = type_index?;
                            validate_type_index(&type_index, self.max_type_index)?;
                            self.update_max_func_index();
                        }
                    },
                    SectionReader::Table(reader) => {
                        for table in reader.clone() {
                            let _table = table?;
                            self.update_max_table_index();
                        }
                    },
                    SectionReader::Memory(reader) => {
                        for memory in reader.clone() {
                            let memory = memory?;
                            validate_memory_type(&memory)?;
                            self.update_max_memory_index();
                        }
                    }
                    SectionReader::Global(reader) => {
                        for global in reader.clone() {
                            let mut global = global?;
                            validate_global_type(&mut global, &self.globals)?;
                        }
                    },
                    SectionReader::Export(reader) => {
                        let mut export_validator = ExportValidator::new();
                        let max_global_index =
                            if self.globals.is_empty() {
                                None
                            } else {
                                Some(GlobalIndex(self.globals.len() as u32 - 1))
                            };
                        for export in reader.clone() {
                            let export = export?;
                            export_validator.validate(
                                &export,
                                self.max_func_index,
                                self.max_table_index,
                                self.max_memory_index,
                                max_global_index,
                            )?;
                        }
                    }
                    _ => {}
                }
            }
            Chunk::Done => {
            }
        }
        Ok(())
    }

    fn add_import_desc(&mut self, import_desc: &ImportDescriptor) {
        match import_desc {
            ImportDescriptor::Func { .. } => {
                self.update_max_func_index();
            }
            ImportDescriptor::Table(_) => {
                self.update_max_table_index();
            }
            ImportDescriptor::Memory(_) => {
                self.update_max_memory_index();
            }
            ImportDescriptor::Global(global_type) => {
                self.globals.push(*global_type)
            }
        }
    }

    fn update_max_func_index(&mut self) {
        self.max_func_index = Some(match self.max_func_index {
            None => { FuncIndex(0) }
            Some(current) => { FuncIndex(current.0 + 1) }
        });
    }

    fn update_max_table_index(&mut self) {
        self.max_table_index = Some(match self.max_table_index {
            None => { TableIndex(0) }
            Some(current) => { TableIndex(current.0 + 1) }
        });
    }

    fn update_max_memory_index(&mut self) {
        self.max_memory_index = Some(match self.max_memory_index {
            None => { MemoryIndex(0) }
            Some(current) => { MemoryIndex(current.0 + 1) }
        });
    }
}