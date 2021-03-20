use crate::{InstructionReader, Instruction, InstructionReaderError};
use crate::types::{ValueType, GlobalType};
use crate::validators::code::CodeValidationError::InvalidInitExpr;

#[derive(PartialEq, Eq, Debug)]
pub enum CodeValidationError {
    InvalidInitExpr,
    InstructionReader(InstructionReaderError)
}

impl From<InstructionReaderError> for CodeValidationError {
    fn from(e: InstructionReaderError) -> Self {
        CodeValidationError::InstructionReader(e)
    }
}

pub fn is_expr_const_and_of_right_type(
    instruction_reader: &mut InstructionReader,
    expected_type: ValueType,
    globals: &[GlobalType]
) -> Result<(), CodeValidationError> {
    let instruction = instruction_reader.read()?;
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

    if const_type != expected_type {
        return Err(InvalidInitExpr);
    }

    let instruction = instruction_reader.read()?;
    match instruction {
        Instruction::End => {},
        _ => return Err(InvalidInitExpr),
    }

    match instruction_reader.read() {
        Ok(_) => {
            return Err(InvalidInitExpr);
        }
        Err(_) => {}
    }

    Ok(())
}
