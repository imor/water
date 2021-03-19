use crate::types::{GlobalSegment, GlobalType};
use crate::{InstructionReaderError, Instruction};
use crate::types::ValueType;
use crate::validators::global::GlobalValidationError::InvalidInitExpr;

#[derive(PartialEq, Eq, Debug)]
pub enum GlobalValidationError {
    InvalidInitExpr,
    InstructionReaderError(InstructionReaderError),
}

impl From<InstructionReaderError> for GlobalValidationError {
    fn from(e: InstructionReaderError) -> Self {
        GlobalValidationError::InstructionReaderError(e)
    }
}

pub fn validate_global_type(global: &mut GlobalSegment, globals: &[GlobalType]) -> Result<(), GlobalValidationError>{
    let instruction = global.instruction_reader.read()?;
    let const_type = match instruction {
        Instruction::I32Const(_) => ValueType::I32,
        Instruction::I64Const(_) => ValueType::I64,
        Instruction::F32Const(_) => ValueType::F32,
        Instruction::F64Const(_) => ValueType::F64,
        Instruction::GlobalGet { global_index } => {
            if let Some(global) = globals.get(global_index.0 as usize) {
                global.var_type
            } else {
                return Err(InvalidInitExpr);
            }
        }
        _ => return Err(InvalidInitExpr),
    };

    if const_type != global.global_type.var_type {
        return Err(InvalidInitExpr);
    }

    let instruction = global.instruction_reader.read()?;
    match instruction {
        Instruction::End => {},
        _ => return Err(InvalidInitExpr),
    }

    match global.instruction_reader.read() {
        Ok(_) => {
            return Err(InvalidInitExpr);
        }
        Err(_) => {}
    }
    Ok(())
}