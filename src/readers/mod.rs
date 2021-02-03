pub use section::code_section::{CodeReaderError, CodeSectionReader};
pub use section::custom_section::{CustomReaderError, CustomSectionReader};
pub use section::data_section::{DataReaderError, DataSectionReader};
pub use section::element_section::{ElementReaderError, ElementSectionReader};
pub use section::export_section::{ExportReaderError, ExportSectionReader};
pub use section::function_section::{FunctionReaderError, FunctionSectionReader};
pub use section::global_section::{GlobalReaderError, GlobalSectionReader};
pub use section::import_section::{ImportReaderError, ImportSectionReader};
pub use section::memory_section::{MemoryReaderError, MemorySectionReader};
pub use section::start_section::{StartReaderError, StartSectionReader};
pub use section::table_section::{TableReaderError, TableSectionReader};
pub use section::type_section::{TypeReaderError, TypeSectionReader};

pub use crate::readers::branch_table::{BranchReaderError, BranchTableReader};
pub use crate::readers::instruction::{InstructionReader, InstructionReaderError};
pub use crate::readers::preamble::{PreambleReader, PreambleReaderError};

//TODO:review what needs to be pub or pub(crate) everywhere
pub(crate) mod preamble;
mod branch_table;
mod instruction;
pub mod binary;
mod section;