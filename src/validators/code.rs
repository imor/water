use crate::{InstructionReader, Instruction, InstructionReaderError};
use crate::types::{ValueType, GlobalType};
use crate::validators::code::CodeValidationError::{InvalidInitExpr, TypeMismatch};
use std::result;

#[derive(PartialEq, Eq, Debug)]
pub enum CodeValidationError {
    InvalidInitExpr,
    InstructionReader(InstructionReaderError),
    TypeMismatch,
}

impl From<InstructionReaderError> for CodeValidationError {
    fn from(e: InstructionReaderError) -> Self {
        CodeValidationError::InstructionReader(e)
    }
}

pub type Result<T, E = CodeValidationError> = result::Result<T, E>;

pub fn is_expr_const_and_of_right_type(
    instruction_reader: &mut InstructionReader,
    expected_type: ValueType,
    globals: &[GlobalType]
) -> Result<()> {
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

    if instruction_reader.read().is_ok() {
        return Err(InvalidInitExpr);
    }

    Ok(())
}

struct ControlFrame {
    label_types: Vec<ValueType>,
    end_types: Vec<ValueType>,
    height: u32,
    unreachable: bool,
}

struct CodeValidator {
    operand_stack: Vec<Option<ValueType>>,
    control_stack: Vec<ControlFrame>,
}

impl CodeValidator {
    fn push_operand(&mut self, operand: Option<ValueType>) {
        self.operand_stack.push(operand);
    }

    fn pop_operand(&mut self) -> Result<Option<ValueType>> {
        //TODO:ensure that unwrap doesn't panic
        let last = self.control_stack.last().unwrap();
        if self.operand_stack.len() == last.height as usize {
            if last.unreachable {
                Ok(None)
            } else {
                Err(TypeMismatch)
            }
        } else {
            Ok(self.operand_stack.pop().unwrap())
        }
    }

    fn pop_expected_operand(&mut self, expected: Option<ValueType>) -> Result<Option<ValueType>> {
        let actual = self.pop_operand()?;
        if actual.is_none() {
            return Ok(expected);
        }

        if expected.is_none() {
            return Ok(actual);
        }

        if actual != expected {
            return Err(TypeMismatch);
        }

        Ok(actual)
    }

    // fn pop_operands(&mut self, expected_types: &Vec<ValueType>) -> Result<()> {
    //     for expected in expected_types.iter().rev() {
    //         self.pop_expected_operand(Some(*expected))?;
    //     }
    //     Ok(())
    // }

    fn push_control_frame(&mut self, label_types: Vec<ValueType>, end_types: Vec<ValueType>) {
        let height = self.operand_stack.len() as u32;
        let frame = ControlFrame { label_types, end_types, height, unreachable: false };
        self.control_stack.push(frame);
    }

    // fn pop_control_frame(&mut self) -> Result<Vec<ValueType>> {
    //     if self.control_stack.is_empty() {
    //         return Err(TypeMismatch);
    //     }
    //
    //     let last = self.control_stack.last().unwrap();
    //     let height = last.height;
    //     //self.pop_operands(&last.end_types)?;//.map::<Vec<ValueType>, dyn FnOnce(()) -> Vec<ValueType>>(|r| Vec::new())?;
    //     for expected in last.end_types.iter().rev() {
    //         self.pop_expected_operand(Some(*expected))?;
    //     }
    //     if self.operand_stack.len() != height as usize {
    //         return Err(TypeMismatch);
    //     }
    //     self.control_stack.pop();
    //     Ok(last.end_types.clone())
    // }

    pub fn validate(&self, instruction: &Instruction) -> Result<()> {
        match instruction {
            Instruction::Unreachable => {}
            Instruction::Nop => {}
            Instruction::Block { .. } => {}
            Instruction::Loop { .. } => {}
            Instruction::If { .. } => {}
            Instruction::Else => {}
            Instruction::End => {}
            Instruction::Branch { .. } => {}
            Instruction::BranchIf { .. } => {}
            Instruction::BranchTable { .. } => {}
            Instruction::Return => {}
            Instruction::Call { .. } => {}
            Instruction::CallIndirect { .. } => {}
            Instruction::Drop => {}
            Instruction::Select => {}
            Instruction::LocalGet { .. } => {}
            Instruction::LocalSet { .. } => {}
            Instruction::LocalTee { .. } => {}
            Instruction::GlobalGet { .. } => {}
            Instruction::GlobalSet { .. } => {}
            Instruction::I32Load { .. } => {}
            Instruction::I64Load { .. } => {}
            Instruction::F32Load { .. } => {}
            Instruction::F64Load { .. } => {}
            Instruction::I32Load8s { .. } => {}
            Instruction::I32Load8u { .. } => {}
            Instruction::I32Load16s { .. } => {}
            Instruction::I32Load16u { .. } => {}
            Instruction::I64Load8s { .. } => {}
            Instruction::I64Load8u { .. } => {}
            Instruction::I64Load16s { .. } => {}
            Instruction::I64Load16u { .. } => {}
            Instruction::I64Load32s { .. } => {}
            Instruction::I64Load32u { .. } => {}
            Instruction::I32Store { .. } => {}
            Instruction::I64Store { .. } => {}
            Instruction::F32Store { .. } => {}
            Instruction::F64Store { .. } => {}
            Instruction::I32Store8 { .. } => {}
            Instruction::I32Store16 { .. } => {}
            Instruction::I64Store8 { .. } => {}
            Instruction::I64Store16 { .. } => {}
            Instruction::I64Store32 { .. } => {}
            Instruction::MemorySize => {}
            Instruction::MemoryGrow => {}
            Instruction::I32Const(_) => {}
            Instruction::I64Const(_) => {}
            Instruction::F32Const(_) => {}
            Instruction::F64Const(_) => {}
            Instruction::I32Eqz => {}
            Instruction::I32Eq => {}
            Instruction::I32Ne => {}
            Instruction::I32Lts => {}
            Instruction::I32Ltu => {}
            Instruction::I32Gts => {}
            Instruction::I32Gtu => {}
            Instruction::I32Les => {}
            Instruction::I32Leu => {}
            Instruction::I32Ges => {}
            Instruction::I32Geu => {}
            Instruction::I64Eqz => {}
            Instruction::I64Eq => {}
            Instruction::I64Ne => {}
            Instruction::I64Lts => {}
            Instruction::I64Ltu => {}
            Instruction::I64Gts => {}
            Instruction::I64Gtu => {}
            Instruction::I64Les => {}
            Instruction::I64Leu => {}
            Instruction::I64Ges => {}
            Instruction::I64Geu => {}
            Instruction::F32Eq => {}
            Instruction::F32Ne => {}
            Instruction::F32Lt => {}
            Instruction::F32Gt => {}
            Instruction::F32Le => {}
            Instruction::F32Ge => {}
            Instruction::F64Eq => {}
            Instruction::F64Ne => {}
            Instruction::F64Lt => {}
            Instruction::F64Gt => {}
            Instruction::F64Le => {}
            Instruction::F64Ge => {}
            Instruction::I32Clz => {}
            Instruction::I32Ctz => {}
            Instruction::I32Popcnt => {}
            Instruction::I32Add => {}
            Instruction::I32Sub => {}
            Instruction::I32Mul => {}
            Instruction::I32Divs => {}
            Instruction::I32Divu => {}
            Instruction::I32Rems => {}
            Instruction::I32Remu => {}
            Instruction::I32And => {}
            Instruction::I32Or => {}
            Instruction::I32Xor => {}
            Instruction::I32Shl => {}
            Instruction::I32Shrs => {}
            Instruction::I32Shru => {}
            Instruction::I32Rotl => {}
            Instruction::I32Rotr => {}
            Instruction::I64Clz => {}
            Instruction::I64Ctz => {}
            Instruction::I64Popcnt => {}
            Instruction::I64Add => {}
            Instruction::I64Sub => {}
            Instruction::I64Mul => {}
            Instruction::I64Divs => {}
            Instruction::I64Divu => {}
            Instruction::I64Rems => {}
            Instruction::I64Remu => {}
            Instruction::I64And => {}
            Instruction::I64Or => {}
            Instruction::I64Xor => {}
            Instruction::I64Shl => {}
            Instruction::I64Shrs => {}
            Instruction::I64Shru => {}
            Instruction::I64Rotl => {}
            Instruction::I64Rotr => {}
            Instruction::F32Abs => {}
            Instruction::F32Neg => {}
            Instruction::F32Ceil => {}
            Instruction::F32Floor => {}
            Instruction::F32Trunc => {}
            Instruction::F32Nearest => {}
            Instruction::F32Sqrt => {}
            Instruction::F32Add => {}
            Instruction::F32Sub => {}
            Instruction::F32Mul => {}
            Instruction::F32Div => {}
            Instruction::F32Min => {}
            Instruction::F32Max => {}
            Instruction::F32Copysign => {}
            Instruction::F64Abs => {}
            Instruction::F64Neg => {}
            Instruction::F64Ceil => {}
            Instruction::F64Floor => {}
            Instruction::F64Trunc => {}
            Instruction::F64Nearest => {}
            Instruction::F64Sqrt => {}
            Instruction::F64Add => {}
            Instruction::F64Sub => {}
            Instruction::F64Mul => {}
            Instruction::F64Div => {}
            Instruction::F64Min => {}
            Instruction::F64Max => {}
            Instruction::F64Copysign => {}
            Instruction::I32WrapI64 => {}
            Instruction::I32TruncF32s => {}
            Instruction::I32TruncF32u => {}
            Instruction::I32TruncF64s => {}
            Instruction::I32TruncF64u => {}
            Instruction::I64ExtendI32s => {}
            Instruction::I64ExtendI32u => {}
            Instruction::I64TruncF32s => {}
            Instruction::I64TruncF32u => {}
            Instruction::I64TruncF64s => {}
            Instruction::I64TruncF64u => {}
            Instruction::F32ConvertI32s => {}
            Instruction::F32ConvertI32u => {}
            Instruction::F32ConvertI64s => {}
            Instruction::F32ConvertI64u => {}
            Instruction::F32DemoteF64 => {}
            Instruction::F64ConvertI32s => {}
            Instruction::F64ConvertI32u => {}
            Instruction::F64ConvertI64s => {}
            Instruction::F64ConvertI64u => {}
            Instruction::F64PromoteF32 => {}
            Instruction::I32ReinterpretF32 => {}
            Instruction::I64ReinterpretF64 => {}
            Instruction::F32ReinterpretI32 => {}
            Instruction::F64ReinterpretI64 => {}
            Instruction::I32Extend8s => {}
            Instruction::I32Extend16s => {}
            Instruction::I64Extend8s => {}
            Instruction::I64Extend16s => {}
            Instruction::I64Extend32s => {}
            Instruction::I32TruncSatF32s => {}
            Instruction::I32TruncSatF32u => {}
            Instruction::I32TruncSatF64s => {}
            Instruction::I32TruncSatF64u => {}
            Instruction::I64TruncSatF32s => {}
            Instruction::I64TruncSatF32u => {}
            Instruction::I64TruncSatF64s => {}
            Instruction::I64TruncSatF64u => {}
        }

        Ok(())
    }
}