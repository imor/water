pub use crate::readers::type_section::{TypeSectionReader, TypeReaderError};
pub use crate::readers::import_section::{ImportSectionReader, ImportReaderError};
pub use crate::readers::function_section::{FunctionSectionReader, FunctionReaderError};

mod type_section;
mod import_section;
mod function_section;