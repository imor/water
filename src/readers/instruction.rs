use crate::binary_reader::{BinaryReader, BinaryReaderError};
use crate::binary_reader::Result as BinaryReaderResult;
use std::result;
use crate::types::{Instruction, BlockType, TypeIndex, LabelIndex, FuncIndex, LocalIndex, GlobalIndex, MemArg};
use crate::readers::instruction::InstructionReaderError::{InvalidInstruction, InvalidBlockTypeIndex, InvalidMemorySizeByte};
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
    InvalidMemorySizeByte,
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

            0x1A => Ok(Instruction::Drop),
            0x1B => Ok(Instruction::Select),

            0x20 => {
                let local_index = LocalIndex(self.reader.read_var_u32()?);
                Ok(Instruction::LocalGet { local_index })
            },
            0x21 => {
                let local_index = LocalIndex(self.reader.read_var_u32()?);
                Ok(Instruction::LocalSet { local_index })
            },
            0x22 => {
                let local_index = LocalIndex(self.reader.read_var_u32()?);
                Ok(Instruction::LocalTee { local_index })
            },
            0x23 => {
                let global_index = GlobalIndex(self.reader.read_var_u32()?);
                Ok(Instruction::GlobalGet { global_index })
            },
            0x24 => {
                let global_index = GlobalIndex(self.reader.read_var_u32()?);
                Ok(Instruction::GlobalSet { global_index })
            },

            0x28 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Load { memarg })
            },
            0x29 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load { memarg })
            },
            0x2A => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::F32Load { memarg })
            },
            0x2B => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::F64Load { memarg })
            },
            0x2C => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Load8s { memarg })
            },
            0x2D => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Load8u { memarg })
            },
            0x2E => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Load16s { memarg })
            },
            0x2F => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Load16u { memarg })
            },
            0x30 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load8s { memarg })
            },
            0x31 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load8u { memarg })
            },
            0x32 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load16s { memarg })
            },
            0x33 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load16u { memarg })
            },
            0x34 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load32s { memarg })
            },
            0x35 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Load32u { memarg })
            },
            0x36 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Store { memarg })
            },
            0x37 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Store { memarg })
            },
            0x38 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::F32Store { memarg })
            },
            0x39 => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::F64Store { memarg })
            },
            0x3A => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Store8 { memarg })
            },
            0x3B => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I32Store16 { memarg })
            },
            0x3C => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Store8 { memarg })
            },
            0x3D => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Store16 { memarg })
            },
            0x3E => {
                let memarg = self.read_memarg()?;
                Ok(Instruction::I64Store32 { memarg })
            },
            0x3F => {
                if let Ok(0x00) = self.reader.read_u8() {
                    Ok(Instruction::MemorySize)
                } else {
                    Err(InvalidMemorySizeByte)
                }
            },
            0x40 => {
                if let Ok(0x00) = self.reader.read_u8() {
                    Ok(Instruction::MemoryGrow)
                } else {
                    Err(InvalidMemorySizeByte)
                }
            },

            _ => Err(InvalidInstruction),
        }
    }

    fn read_memarg(&mut self) -> Result<MemArg> {
        let alignment = self.reader.read_var_u32()?;
        let offset = self.reader.read_var_u32()?;
        Ok(MemArg { alignment, offset })
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
