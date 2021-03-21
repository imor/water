pub use section::code::{CodeReaderError, CodeSectionReader};
pub use section::custom::{CustomReaderError, CustomSectionReader};
pub use section::data::{DataReaderError, DataSectionReader};
pub use section::element::{ElementReaderError, ElementSectionReader};
pub use section::export::{ExportReaderError, ExportSectionReader};
pub use section::function::{FunctionReaderError, FunctionSectionReader};
pub use section::global::{GlobalReaderError, GlobalSectionReader};
pub use section::import::{ImportReaderError, ImportSectionReader};
pub use section::memory::{MemoryReaderError, MemorySectionReader};
pub use section::start::{StartReaderError, StartSectionReader};
pub use section::table::{TableReaderError, TableSectionReader};
pub use section::r#type::{TypeReaderError, TypeSectionReader};

pub use crate::readers::branch_table::{BranchReaderError, BranchTableReader};
pub use crate::readers::instruction::{InstructionReader, InstructionReaderError};
pub use crate::readers::preamble::{PreambleReader, PreambleReaderError};

//TODO:review what needs to be pub or pub(crate) everywhere
pub(crate) mod preamble;
mod branch_table;
mod instruction;
pub mod binary;
pub mod section;
mod common;