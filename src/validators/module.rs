use crate::{Chunk, SectionReader, ImportReaderError, FunctionReaderError, TableReaderError, MemoryReaderError, GlobalReaderError, ExportReaderError, TypeReaderError, ElementReaderError, DataReaderError, CodeReaderError, InstructionReaderError};
use std::result;
use crate::validators::preamble::{validate_preamble, PreambleValidationError};
use crate::validators::import::{validate_import_desc, ImportValidationError};
use crate::validators::type_index::{validate_type_index, TypeIndexValidationError};
use crate::types::{TypeIndex, GlobalType, ImportDescriptor, FuncIndex, TableIndex, MemoryIndex, GlobalIndex, FunctionType, TableType, MemoryType};
use crate::validators::memory::{validate_memory_type, MemoryLimitsValidationError};
use crate::validators::global::{validate_global_type, GlobalValidationError};
use crate::validators::export::{ExportValidator, ExportValidationError};
use crate::validators::start::{validate_start, StartValidationError};
use crate::validators::element::{validate_element, ElementValidationError};
use crate::validators::data::{validate_data, DataValidationError};
use crate::ValidationError::UnknownSection;
use crate::validators::code::{CodeValidator, CodeValidationError};

pub struct Validator {
    context: ValidationContext,
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
    CodeReader(CodeReaderError),
    InstructionReader(InstructionReaderError),
    CodeValidation(CodeValidationError),
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

impl From<CodeReaderError> for ValidationError {
    fn from(e: CodeReaderError) -> Self {
        ValidationError::CodeReader(e)
    }
}

impl From<InstructionReaderError> for ValidationError {
    fn from(e: InstructionReaderError) -> Self {
        ValidationError::InstructionReader(e)
    }
}

impl From<CodeValidationError> for ValidationError {
    fn from(e: CodeValidationError) -> Self {
        ValidationError::CodeValidation(e)
    }
}

pub type Result<T, E = ValidationError> = result::Result<T, E>;

struct ValidationContext {
    function_types: Vec<FunctionType>,
    globals: Vec<GlobalType>,
    function_type_indices: Vec<TypeIndex>,
    max_table_index: Option<TableIndex>,
    max_memory_index: Option<MemoryIndex>,
}

impl ValidationContext {
    fn new() -> ValidationContext {
        ValidationContext {
            function_types: Vec::new(),
            globals: Vec::new(),
            function_type_indices: Vec::new(),
            max_table_index: None,
            max_memory_index: None,
        }
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

    fn get_max_table_index(&self) -> Option<TableIndex> {
        self.max_table_index
    }

    fn get_max_memory_index(&self) -> Option<MemoryIndex> {
        self.max_memory_index
    }

    fn add_import_desc(&mut self, import_desc: &ImportDescriptor) {
        match import_desc {
            ImportDescriptor::Func { type_index: _ } => {
                //self.function_type_indices.push(*type_index);
            }
            ImportDescriptor::Table(table_type) => {
                self.add_table_type(table_type);
            }
            ImportDescriptor::Memory(memory_type) => {
                self.add_memory_type(memory_type);
            }
            ImportDescriptor::Global(global_type) => {
                self.add_global_type(global_type);
            }
        }
    }

    fn add_table_type(&mut self, _table_type: &TableType) {
        self.max_table_index = Some(match self.max_table_index {
            None => { TableIndex(0) }
            Some(current) => { TableIndex(current.0 + 1) }
        });
    }

    fn add_memory_type(&mut self, _memory_type: &MemoryType) {
        self.max_memory_index = Some(match self.max_memory_index {
            None => { MemoryIndex(0) }
            Some(current) => { MemoryIndex(current.0 + 1) }
        });
    }

    fn add_function_type(&mut self, function_type: FunctionType) {
        self.function_types.push(function_type);
    }

    fn add_type_index(&mut self, type_index: TypeIndex) {
        self.function_type_indices.push(type_index);
    }

    fn add_global_type(&mut self, global_type: &GlobalType) {
        self.globals.push(*global_type)
    }
}

impl Validator {
    pub fn new() -> Validator {
        Validator {
            context: ValidationContext::new()
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
                        for function_type in reader.clone() {
                            let function_type = function_type?;
                            self.context.add_function_type(function_type);
                        }
                    },
                    SectionReader::Import(reader) => {
                        for import in reader.clone() {
                            let import = import?;
                            let import_desc = import.import_descriptor;
                            validate_import_desc(&import_desc, self.context.get_max_type_index())?;
                            self.context.add_import_desc(&import_desc);
                        }
                    },
                    SectionReader::Function(reader) => {
                        for type_index in reader.clone() {
                            let type_index = type_index?;
                            validate_type_index(&type_index, self.context.get_max_type_index())?;
                            self.context.add_type_index(type_index);
                        }
                    },
                    SectionReader::Table(reader) => {
                        for table in reader.clone() {
                            let table = table?;
                            self.context.add_table_type(&table);
                        }
                    },
                    SectionReader::Memory(reader) => {
                        for memory_type in reader.clone() {
                            let memory_type = memory_type?;
                            validate_memory_type(&memory_type)?;
                            self.context.add_memory_type(&memory_type);
                        }
                    },
                    SectionReader::Global(reader) => {
                        for global in reader.clone() {
                            let mut global = global?;
                            validate_global_type(&mut global, &self.context.globals)?;
                            self.context.add_global_type(&global.global_type);
                        }
                    },
                    SectionReader::Export(reader) => {
                        let mut export_validator = ExportValidator::new();
                        for export in reader.clone() {
                            let export = export?;
                            export_validator.validate(
                                &export,
                                self.context.get_max_function_index(),
                                self.context.get_max_table_index(),
                                self.context.get_max_memory_index(),
                                self.context.get_max_global_index(),
                            )?;
                        }
                    },
                    SectionReader::Start(reader) => {
                        let func_index = reader.get_func_index();
                        validate_start(func_index, &self.context.function_type_indices, &self.context.function_types)?;
                    },
                    SectionReader::Element(reader) => {
                        for element_segment in reader.clone() {
                            let mut element_segment = element_segment?;
                            validate_element(
                                &mut element_segment,
                                self.context.max_table_index,
                                self.context.get_max_function_index(),
                                &self.context.globals
                            )?;
                        }
                    },
                    SectionReader::Code(reader) => {
                        let mut function_index = 0u32;
                        for code in reader.clone() {
                            let code = code?;

                            let mut code_validator = CodeValidator::new(code);
                            code_validator.validate(
                                &self.context.globals,
                                &self.context.function_types,
                                &self.context.function_type_indices,
                                FuncIndex(function_index),
                                self.context.get_max_table_index(),
                                self.context.get_max_memory_index(),
                            )?;
                            function_index += 1;
                        }
                    }
                    SectionReader::Data(reader) => {
                        for data_segment in reader.clone() {
                            let mut data_segment = data_segment?;
                            validate_data(
                                &mut data_segment,
                                self.context.get_max_memory_index(),
                                &self.context.globals
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
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}