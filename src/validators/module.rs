use crate::{Chunk, SectionReader, ImportReaderError, FunctionReaderError, TableReaderError, MemoryReaderError, GlobalReaderError, ExportReaderError, TypeReaderError, ElementReaderError, DataReaderError};
use std::result;
use crate::validators::preamble::{validate_preamble, PreambleValidationError};
use crate::validators::import::{validate_import_desc, ImportValidationError};
use crate::validators::type_index::{validate_type_index, TypeIndexValidationError};
use crate::types::{TypeIndex, GlobalType, ImportDescriptor, FuncIndex, TableIndex, MemoryIndex, GlobalIndex, FunctionType};
use crate::validators::memory::{validate_memory_type, MemoryLimitsValidationError};
use crate::validators::global::{validate_global_type, GlobalValidationError};
use crate::validators::export::{ExportValidator, ExportValidationError};
use crate::validators::start::{validate_start, StartValidationError};
use crate::validators::element::{validate_element, ElementValidationError};
use crate::validators::data::{validate_data, DataValidationError};
use crate::ValidationError::UnknownSection;

pub struct Validator {
    function_types: Vec<FunctionType>,
    globals: Vec<GlobalType>,
    function_type_indices: Vec<TypeIndex>,
    max_table_index: Option<TableIndex>,
    max_memory_index: Option<MemoryIndex>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ValidationError {
    PreambleValidation(PreambleValidationError),
    TypeReader(TypeReaderError),
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
    StartValidation(StartValidationError),
    ElementReader(ElementReaderError),
    ElementValidation(ElementValidationError),
    DataReader(DataReaderError),
    DataValidation(DataValidationError),
    UnknownSection(u8),
}

impl From<PreambleValidationError> for ValidationError {
    fn from(e: PreambleValidationError) -> Self {
        ValidationError::PreambleValidation(e)
    }
}

impl From<TypeReaderError> for ValidationError {
    fn from(e: TypeReaderError) -> Self {
        ValidationError::TypeReader(e)
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

impl From<StartValidationError> for ValidationError {
    fn from(e: StartValidationError) -> Self {
        ValidationError::StartValidation(e)
    }
}

impl From<ElementReaderError> for ValidationError {
    fn from(e: ElementReaderError) -> Self {
        ValidationError::ElementReader(e)
    }
}

impl From<ElementValidationError> for ValidationError {
    fn from(e: ElementValidationError) -> Self {
        ValidationError::ElementValidation(e)
    }
}

impl From<DataReaderError> for ValidationError {
    fn from(e: DataReaderError) -> Self {
        ValidationError::DataReader(e)
    }
}

impl From<DataValidationError> for ValidationError {
    fn from(e: DataValidationError) -> Self {
        ValidationError::DataValidation(e)
    }
}

pub type Result<T, E = ValidationError> = result::Result<T, E>;

impl Validator {
    pub fn new() -> Validator {
        Validator {
            function_types: Vec::new(),
            globals: Vec::new(),
            function_type_indices: Vec::new(),
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
                    SectionReader::Custom(_) => {}
                    SectionReader::Type(reader) => {
                        for func_type in reader.clone() {
                            let func_type = func_type?;
                            self.function_types.push(func_type);
                        }
                    },
                    SectionReader::Import(reader) => {
                        for import in reader.clone() {
                            let import = import?;
                            let import_desc = import.import_descriptor;
                            validate_import_desc(&import_desc, self.get_max_type_index())?;
                            self.add_import_desc(&import_desc);
                        }
                    },
                    SectionReader::Function(reader) => {
                        for type_index in reader.clone() {
                            let type_index = type_index?;
                            validate_type_index(&type_index, self.get_max_type_index())?;
                            self.function_type_indices.push(type_index);
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
                    },
                    SectionReader::Global(reader) => {
                        for global in reader.clone() {
                            let mut global = global?;
                            validate_global_type(&mut global, &self.globals)?;
                        }
                    },
                    SectionReader::Export(reader) => {
                        let mut export_validator = ExportValidator::new();
                        for export in reader.clone() {
                            let export = export?;
                            export_validator.validate(
                                &export,
                                self.get_max_function_index(),
                                self.max_table_index,
                                self.max_memory_index,
                                self.get_max_global_index(),
                            )?;
                        }
                    },
                    SectionReader::Start(reader) => {
                        let func_index = reader.get_func_index();
                        validate_start(func_index, &self.function_type_indices, &self.function_types)?;
                    },
                    SectionReader::Element(reader) => {
                        for element_segment in reader.clone() {
                            let mut element_segment = element_segment?;
                            validate_element(
                                &mut element_segment,
                                self.max_table_index,
                                self.get_max_function_index(),
                                &self.globals
                            )?;
                        }
                    },
                    SectionReader::Code(_reader) => {

                    }
                    SectionReader::Data(reader) => {
                        for data_segment in reader.clone() {
                            let mut data_segment = data_segment?;
                            validate_data(
                                &mut data_segment,
                                self.max_memory_index,
                                &self.globals
                            )?;
                        }
                    }
                    SectionReader::Unknown(id) => {
                        return Err(UnknownSection(*id));
                    }
                }
            }
            Chunk::Done => {
            }
        }
        Ok(())
    }

    fn get_max_type_index(&self) -> Option<TypeIndex> {
        if self.function_types.is_empty() {
            None
        } else {
            Some(TypeIndex(self.function_types.len() as u32 - 1))
        }
    }

    fn get_max_global_index(&self) -> Option<GlobalIndex> {
        if self.globals.is_empty() {
            None
        } else {
            Some(GlobalIndex(self.globals.len() as u32 - 1))
        }
    }

    fn get_max_function_index(&self) -> Option<FuncIndex> {
        if self.function_type_indices.is_empty() {
            None
        } else {
            Some(FuncIndex(self.function_type_indices.len() as u32 - 1))
        }
    }

    fn add_import_desc(&mut self, import_desc: &ImportDescriptor) {
        match import_desc {
            ImportDescriptor::Func { type_index } => {
                self.function_type_indices.push(*type_index);
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

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}