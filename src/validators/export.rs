use crate::types::{Export, ExportDescriptor, FuncIndex, TableIndex, MemoryIndex, GlobalIndex};
use std::collections::HashSet;
use crate::validators::export::ExportValidationError::{DuplicateName, InvalidFuncIndex, InvalidTableIndex, InvalidMemoryIndex, InvalidGlobalIndex};

#[derive(PartialEq, Eq, Debug)]
pub enum ExportValidationError {
    DuplicateName,
    InvalidFuncIndex(FuncIndex),
    InvalidTableIndex(TableIndex),
    InvalidMemoryIndex(MemoryIndex),
    InvalidGlobalIndex(GlobalIndex),
}

pub struct ExportValidator {
    exported_names: HashSet<String>,
}

impl ExportValidator {
    pub fn new() -> ExportValidator {
        ExportValidator { exported_names: HashSet::new() }
    }

    pub fn validate(&mut self,
                    export: &Export,
                    max_func_index: Option<FuncIndex>,
                    max_table_index: Option<TableIndex>,
                    max_memory_index: Option<MemoryIndex>,
                    max_global_index: Option<GlobalIndex>,
    ) -> Result<(), ExportValidationError> {
        if !self.exported_names.insert(export.name.to_string()) {
            return Err(DuplicateName);
        }
        match export.export_descriptor {
            ExportDescriptor::Func { func_index } => {
                if max_func_index.is_none() || func_index > max_func_index.unwrap() {
                    return Err(InvalidFuncIndex(func_index));
                }
            }
            ExportDescriptor::Table { table_index } => {
                if max_table_index.is_none() || table_index > max_table_index.unwrap() {
                    return Err(InvalidTableIndex(table_index));
                }
            }
            ExportDescriptor::Memory { memory_index } => {
                if max_memory_index.is_none() || memory_index > max_memory_index.unwrap() {
                    return Err(InvalidMemoryIndex(memory_index));
                }
            }
            ExportDescriptor::Global { global_index } => {
                if max_global_index.is_none() || global_index > max_global_index.unwrap() {
                    return Err(InvalidGlobalIndex(global_index));
                }
            }
        }
        Ok(())
    }
}