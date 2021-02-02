use crate::readers::binary::{BinaryReader, BinaryReaderError};
use crate::readers::binary::Result as BinaryReaderResult;
use std::result;
use crate::types::{Instruction, BlockType, TypeIndex, LabelIndex, FuncIndex, LocalIndex, GlobalIndex, MemoryArgument};
use crate::readers::instruction::InstructionReaderError::{InvalidInstruction, InvalidBlockTypeIndex, InvalidMemorySizeByte, InvalidSatOpCode};
use crate::types::Instruction::*;

#[derive(Eq, PartialEq, Debug)]
pub struct InstructionReader<'a> {
    reader: BinaryReader<'a>,
}

#[derive(Debug)]
pub enum InstructionReaderError {
    BinaryReaderError(BinaryReaderError),
    InvalidInstruction,
    InvalidBlockTypeIndex,
    InvalidMemorySizeByte,
    InvalidSatOpCode,
}

impl From<BinaryReaderError> for InstructionReaderError {
    fn from(e: BinaryReaderError) -> Self {
        InstructionReaderError::BinaryReaderError(e)
    }
}

pub type Result<T, E = InstructionReaderError> = result::Result<T, E>;

impl<'a> InstructionReader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> BinaryReaderResult<InstructionReader<'a>> {
        let reader = BinaryReader::new(buffer);
        Ok(InstructionReader { reader })
    }

    pub fn eof(&self) -> bool {
        self.reader.eof()
    }

    pub fn read(&mut self) -> Result<Instruction> {
        match self.reader.read_byte()? {
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
                let label_index = LabelIndex(self.reader.read_u32()?);
                Ok(Instruction::Branch { label_index })
            },
            0x0D => {
                let label_index = LabelIndex(self.reader.read_u32()?);
                Ok(Instruction::BranchIf { label_index })
            },
            0x0E => {
                let branch_table_reader = self.reader.create_branch_table_reader()?;
                Ok(Instruction::BranchTable { branch_table_reader })
            },
            0x0F => Ok(Instruction::Return),
            0x10 => {
                let func_index = FuncIndex(self.reader.read_u32()?);
                Ok(Instruction::Call { func_index })
            },
            0x11 => {
                let type_index = TypeIndex(self.reader.read_u32()?);
                Ok(Instruction::CallIndirect { type_index })
            },

            0x1A => Ok(Instruction::Drop),
            0x1B => Ok(Instruction::Select),

            0x20 => {
                let local_index = LocalIndex(self.reader.read_u32()?);
                Ok(Instruction::LocalGet { local_index })
            },
            0x21 => {
                let local_index = LocalIndex(self.reader.read_u32()?);
                Ok(Instruction::LocalSet { local_index })
            },
            0x22 => {
                let local_index = LocalIndex(self.reader.read_u32()?);
                Ok(Instruction::LocalTee { local_index })
            },
            0x23 => {
                let global_index = GlobalIndex(self.reader.read_u32()?);
                Ok(Instruction::GlobalGet { global_index })
            },
            0x24 => {
                let global_index = GlobalIndex(self.reader.read_u32()?);
                Ok(Instruction::GlobalSet { global_index })
            },

            0x28 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Load { memory_argument })
            },
            0x29 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load { memory_argument })
            },
            0x2A => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::F32Load { memory_argument })
            },
            0x2B => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::F64Load { memory_argument })
            },
            0x2C => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Load8s { memory_argument })
            },
            0x2D => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Load8u { memory_argument })
            },
            0x2E => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Load16s { memory_argument })
            },
            0x2F => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Load16u { memory_argument })
            },
            0x30 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load8s { memory_argument })
            },
            0x31 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load8u { memory_argument })
            },
            0x32 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load16s { memory_argument })
            },
            0x33 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load16u { memory_argument })
            },
            0x34 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load32s { memory_argument })
            },
            0x35 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Load32u { memory_argument })
            },
            0x36 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Store { memory_argument })
            },
            0x37 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Store { memory_argument })
            },
            0x38 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::F32Store { memory_argument })
            },
            0x39 => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::F64Store { memory_argument })
            },
            0x3A => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Store8 { memory_argument })
            },
            0x3B => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I32Store16 { memory_argument })
            },
            0x3C => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Store8 { memory_argument })
            },
            0x3D => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Store16 { memory_argument })
            },
            0x3E => {
                let memory_argument = self.read_memory_argument()?;
                Ok(Instruction::I64Store32 { memory_argument })
            },
            0x3F => {
                if let Ok(0x00) = self.reader.read_byte() {
                    Ok(Instruction::MemorySize)
                } else {
                    Err(InvalidMemorySizeByte)
                }
            },
            0x40 => {
                if let Ok(0x00) = self.reader.read_byte() {
                    Ok(Instruction::MemoryGrow)
                } else {
                    Err(InvalidMemorySizeByte)
                }
            },
            0x41 => {
                let val = self.reader.read_s32()?;
                Ok(Instruction::I32Const(val))
            },
            0x42 => {
                let val = self.reader.read_s64()?;
                Ok(Instruction::I64Const(val))
            },
            0x43 => {
                let val = self.reader.read_f32()?;
                Ok(Instruction::F32Const(val))
            },
            0x44 => {
                let val = self.reader.read_f64()?;
                Ok(Instruction::F64Const(val))
            },

            0x45 => Ok(Instruction::I32Eqz),
            0x46 => Ok(Instruction::I32Eq),
            0x47 => Ok(Instruction::I32Ne),
            0x48 => Ok(Instruction::I32Lts),
            0x49 => Ok(Instruction::I32Ltu),
            0x4A => Ok(Instruction::I32Gts),
            0x4B => Ok(Instruction::I32Gtu),
            0x4C => Ok(Instruction::I32Les),
            0x4D => Ok(Instruction::I32Leu),
            0x4E => Ok(Instruction::I32Ges),
            0x4F => Ok(Instruction::I32Geu),

            0x50 => Ok(Instruction::I64Eqz),
            0x51 => Ok(Instruction::I64Eq),
            0x52 => Ok(Instruction::I64Ne),
            0x53 => Ok(Instruction::I64Lts),
            0x54 => Ok(Instruction::I64Ltu),
            0x55 => Ok(Instruction::I64Gts),
            0x56 => Ok(Instruction::I64Gtu),
            0x57 => Ok(Instruction::I64Les),
            0x58 => Ok(Instruction::I64Leu),
            0x59 => Ok(Instruction::I64Ges),
            0x5A => Ok(Instruction::I64Geu),

            0x5B => Ok(Instruction::F32Eq),
            0x5C => Ok(Instruction::F32Ne),
            0x5D => Ok(Instruction::F32Lt),
            0x5E => Ok(Instruction::F32Gt),
            0x5F => Ok(Instruction::F32Le),
            0x60 => Ok(Instruction::F32Ge),

            0x61 => Ok(Instruction::F64Eq),
            0x62 => Ok(Instruction::F64Ne),
            0x63 => Ok(Instruction::F64Lt),
            0x64 => Ok(Instruction::F64Gt),
            0x65 => Ok(Instruction::F64Le),
            0x66 => Ok(Instruction::F64Ge),

            0x67 => Ok(Instruction::I32Clz),
            0x68 => Ok(Instruction::I32Ctz),
            0x69 => Ok(Instruction::I32Popcnt),
            0x6A => Ok(Instruction::I32Add),
            0x6B => Ok(Instruction::I32Sub),
            0x6C => Ok(Instruction::I32Mul),
            0x6D => Ok(Instruction::I32Divs),
            0x6E => Ok(Instruction::I32Divu),
            0x6F => Ok(Instruction::I32Rems),
            0x70 => Ok(Instruction::I32Remu),
            0x71 => Ok(Instruction::I32And),
            0x72 => Ok(Instruction::I32Or),
            0x73 => Ok(Instruction::I32Xor),
            0x74 => Ok(Instruction::I32Shl),
            0x75 => Ok(Instruction::I32Shrs),
            0x76 => Ok(Instruction::I32Shru),
            0x77 => Ok(Instruction::I32Rotl),
            0x78 => Ok(Instruction::I32Rotr),

            0x79 => Ok(Instruction::I64Clz),
            0x7A => Ok(Instruction::I64Ctz),
            0x7B => Ok(Instruction::I64Popcnt),
            0x7C => Ok(Instruction::I64Add),
            0x7D => Ok(Instruction::I64Sub),
            0x7E => Ok(Instruction::I64Mul),
            0x7F => Ok(Instruction::I64Divs),
            0x80 => Ok(Instruction::I64Divu),
            0x81 => Ok(Instruction::I64Rems),
            0x82 => Ok(Instruction::I64Remu),
            0x83 => Ok(Instruction::I64And),
            0x84 => Ok(Instruction::I64Or),
            0x85 => Ok(Instruction::I64Xor),
            0x86 => Ok(Instruction::I64Shl),
            0x87 => Ok(Instruction::I64Shrs),
            0x88 => Ok(Instruction::I64Shru),
            0x89 => Ok(Instruction::I64Rotl),
            0x8A => Ok(Instruction::I64Rotr),

            0x8B => Ok(Instruction::F32Abs),
            0x8C => Ok(Instruction::F32Neg),
            0x8D => Ok(Instruction::F32Ceil),
            0x8E => Ok(Instruction::F32Floor),
            0x8F => Ok(Instruction::F32Trunc),
            0x90 => Ok(Instruction::F32Nearest),
            0x91 => Ok(Instruction::F32Sqrt),
            0x92 => Ok(Instruction::F32Add),
            0x93 => Ok(Instruction::F32Sub),
            0x94 => Ok(Instruction::F32Mul),
            0x95 => Ok(Instruction::F32Div),
            0x96 => Ok(Instruction::F32Min),
            0x97 => Ok(Instruction::F32Max),
            0x98 => Ok(Instruction::F32Copysign),

            0x99 => Ok(Instruction::F64Abs),
            0x9A => Ok(Instruction::F64Neg),
            0x9B => Ok(Instruction::F64Ceil),
            0x9C => Ok(Instruction::F64Floor),
            0x9D => Ok(Instruction::F64Trunc),
            0x9E => Ok(Instruction::F64Nearest),
            0x9F => Ok(Instruction::F64Sqrt),
            0xA0 => Ok(Instruction::F64Add),
            0xA1 => Ok(Instruction::F64Sub),
            0xA2 => Ok(Instruction::F64Mul),
            0xA3 => Ok(Instruction::F64Div),
            0xA4 => Ok(Instruction::F64Min),
            0xA5 => Ok(Instruction::F64Max),
            0xA6 => Ok(Instruction::F64Copysign),

            0xA7 => Ok(Instruction::I32WrapI64),
            0xA8 => Ok(Instruction::I32TruncF32s),
            0xA9 => Ok(Instruction::I32TruncF32u),
            0xAA => Ok(Instruction::I32TruncF64s),
            0xAB => Ok(Instruction::I32TruncF64u),
            0xAC => Ok(Instruction::I64ExtendI32s),
            0xAD => Ok(Instruction::I64ExtendI32u),
            0xAE => Ok(Instruction::I64TruncF32s),
            0xAF => Ok(Instruction::I64TruncF32u),
            0xB0 => Ok(Instruction::I64TruncF64s),
            0xB1 => Ok(Instruction::I64TruncF64u),
            0xB2 => Ok(Instruction::F32ConvertI32s),
            0xB3 => Ok(Instruction::F32ConvertI32u),
            0xB4 => Ok(Instruction::F32ConvertI64s),
            0xB5 => Ok(Instruction::F32ConvertI64u),
            0xB6 => Ok(Instruction::F32DemoteF64),
            0xB7 => Ok(Instruction::F64ConvertI32s),
            0xB8 => Ok(Instruction::F64ConvertI32u),
            0xB9 => Ok(Instruction::F64ConvertI64s),
            0xBA => Ok(Instruction::F64ConvertI64u),
            0xBB => Ok(Instruction::F64PromoteF32),
            0xBC => Ok(Instruction::I32ReinterpretF32),
            0xBD => Ok(Instruction::I64ReinterpretF64),
            0xBE => Ok(Instruction::F32ReinterpretI32),
            0xBF => Ok(Instruction::F64ReinterpretI64),

            0xC0 => Ok(Instruction::I32Extend8s),
            0xC1 => Ok(Instruction::I32Extend16s),
            0xC2 => Ok(Instruction::I64Extend8s),
            0xC3 => Ok(Instruction::I64Extend16s),
            0xC4 => Ok(Instruction::I64Extend32s),

            0xFC => {
                match self.reader.read_u32()? {
                    0 => Ok(Instruction::I32TruncSatF32s),
                    1 => Ok(Instruction::I32TruncSatF32u),
                    2 => Ok(Instruction::I32TruncSatF64s),
                    3 => Ok(Instruction::I32TruncSatF64u),
                    4 => Ok(Instruction::I64TruncSatF32s),
                    5 => Ok(Instruction::I64TruncSatF32u),
                    6 => Ok(Instruction::I64TruncSatF64s),
                    7 => Ok(Instruction::I64TruncSatF64u),
                    _ => Err(InvalidSatOpCode)
                }
            }

            _ => Err(InvalidInstruction),
        }
    }

    fn read_memory_argument(&mut self) -> Result<MemoryArgument> {
        let alignment = self.reader.read_u32()?;
        let offset = self.reader.read_u32()?;
        Ok(MemoryArgument { alignment, offset })
    }

    fn read_block_type(&mut self) -> Result<BlockType> {
        let position = self.reader.position;
        if let Ok(val_type) = self.reader.read_value_type() {
            Ok(BlockType::ValueType(val_type))
        } else {
            self.reader.position = position;
            match self.reader.read_byte()? {
                0x40 => Ok(BlockType::Empty),
                _ => {
                    let index = self.reader.read_s33()?;
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
