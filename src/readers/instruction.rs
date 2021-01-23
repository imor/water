use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::{Instruction, BlockType, TypeIndex, LabelIndex, FuncIndex};
use crate::readers::instruction::InstructionReaderError::{InvalidInstruction, InvalidBlockTypeIndex};
use crate::types::Instruction::*;

#[derive(Eq, PartialEq, Debug)]
pub struct InstructionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

#[derive(Debug)]
pub enum InstructionReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidInstruction,
    InvalidBlockTypeIndex,
}

impl From<BinaryReaderError> for InstructionReaderError {
    fn from(e: BinaryReaderError) -> Self {
        InstructionReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = InstructionReaderError> = result::Result<T, E>;

impl<'a> InstructionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<InstructionReader<'a>> {
        let mut reader = BinaryReader::new(buffer);
        let count = reader.read_var_u32()?;
        Ok(InstructionReader { reader, count })
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn read(&mut self) -> Result<Instruction> {
        match self.reader.read_u8()? {
            0x00 => Ok(Unreachable),
            0x01 => Ok(Nop),
            0x02 => {
                let block_type = self.read_block_type()?;
                Ok(Instruction::Block { block_type })
            },
            0x03 => {
                let block_type = self.read_block_type()?;
                Ok(Instruction::Loop { block_type })
            },
            0x04 => {
                let block_type = self.read_block_type()?;
                Ok(Instruction::If { block_type })
            },
            0x05 => Ok(Instruction::Else),
            0x0B => Ok(Instruction::End),
            0x0C => {
                let label_index = LabelIndex(self.reader.read_var_u32()?);
                Ok(Instruction::Branch { label_index })
            },
            0x0D => {
                let label_index = LabelIndex(self.reader.read_var_u32()?);
                Ok(Instruction::BranchIf { label_index })
            },
            0x0E => {
                let branch_table_reader = self.reader.create_branch_table_reader()?;
                Ok(Instruction::BranchTable { branch_table_reader })
            },
            0x0F => Ok(Instruction::Return),
            0x10 => {
                let func_index = FuncIndex(self.reader.read_var_u32()?);
                Ok(Instruction::Call { func_index })
            },
            0x11 => {
                let type_index = TypeIndex(self.reader.read_var_u32()?);
                Ok(Instruction::CallIndirect { type_index })
            },
            _ => Err(InvalidInstruction),
        }
    }

    fn read_block_type(&mut self) -> Result<BlockType> {
        if let Ok(val_type) = self.reader.read_value_type() {
            Ok(BlockType::ValueType(val_type))
        } else {
            match self.reader.read_u8()? {
                0x40 => Ok(BlockType::Empty),
                _ => {
                    let index = self.reader.read_var_s33()?;
                    if index < 0 || index > u32::max_value() as i64 {
                        Err(InvalidBlockTypeIndex)
                    } else {
                        Ok(BlockType::TypeIndex(TypeIndex(index as u32)))
                    }
                }
            }
        }
    }
}
