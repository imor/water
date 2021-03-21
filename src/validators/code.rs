use crate::{InstructionReader, Instruction, InstructionReaderError, CodeReaderError};
use crate::types::{ValueType, GlobalType, GlobalIndex, LocalIndex, TypeIndex, FuncIndex, Locals, FunctionType};
use crate::validators::code::CodeValidationError::{InvalidInitExpr, TypeMismatch, InvalidGlobalIndex, InvalidLocalIndex, InvalidTypeIndex, InvalidFunctionIndex, SettingImmutableGlobal};
use std::result;
use crate::readers::section::code::{Code, LocalsReader, LocalsIterationProof};
use crate::validators::code::Operand::{Unknown, Known};

#[derive(PartialEq, Eq, Debug)]
pub enum CodeValidationError {
    CodeReader(CodeReaderError),
    InvalidInitExpr,
    InstructionReader(InstructionReaderError),
    InvalidGlobalIndex(GlobalIndex),
    SettingImmutableGlobal(GlobalIndex),
    InvalidLocalIndex(LocalIndex),
    InvalidTypeIndex(TypeIndex),
    InvalidFunctionIndex(FuncIndex),
    TypeMismatch,
}

impl From<CodeReaderError> for CodeValidationError {
    fn from(e: CodeReaderError) -> Self {
        CodeValidationError::CodeReader(e)
    }
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
    height: usize,
    unreachable: bool,
}

pub struct CodeValidator<'a> {
    code: Code<'a>,
}

impl<'a> CodeValidator<'a> {
    pub fn new(code: Code<'a>) -> CodeValidator<'a> {
        CodeValidator { code }
    }

    pub fn validate(&mut self,
                    globals: &[GlobalType],
                    function_types: &[FunctionType],
                    function_type_indices: &[TypeIndex],
                    function_index: FuncIndex
    ) -> Result<()> {
        let mut state = CodeValidatorState::new();
        let locals_reader = self.code.get_locals_reader()?;
        let (locals, locals_iteration_proof) = self.create_locals(
            locals_reader,
            function_types,
            function_type_indices,
            function_index
        )?;
        let instruction_reader = self.code.get_instruction_reader(locals_iteration_proof)?;

        for instruction in instruction_reader {
            let instruction = instruction?;
            println!("Validating instruction: {:?}", instruction);
            state.validate_instruction(&instruction, globals, &locals)?;
        }
        Ok(())
    }

    fn create_locals(&self,
                     mut locals_reader: LocalsReader,
                     function_types: &[FunctionType],
                     function_type_indices: &[TypeIndex],
                     function_index: FuncIndex
    ) -> Result<(Vec<ValueType>, LocalsIterationProof)> {
        //TODO:For now we are creating a vec of locals,
        //this can be represented more compactly which allows binary search
        //over that compressed representation. Use that representation.
        let mut locals = Vec::new();
        let params = if let Some(func_type_index) = function_type_indices.get(function_index.0 as usize) {
            if let Some(function_type) = function_types.get(func_type_index.0 as usize) {
                &function_type.params
            } else {
                return Err(InvalidTypeIndex(*func_type_index));
            }
        } else {
            return Err(InvalidFunctionIndex(function_index));
        };
        for param in params.into_iter() {
            locals.push(*param);
        }
        let locals_results: Vec<Result<Locals, CodeReaderError>> = locals_reader.into_iter().collect();
        for local in locals_results {
            let local = local?;
            for _ in 0..local.count {
                locals.push(local.value_type);
            }
        }
        Ok((locals, locals_reader.get_iteration_proof()?))
    }

}

#[derive(Eq, PartialEq)]
enum Operand {
    Known(ValueType),
    Unknown,
}

impl Operand {
    fn is_known(&self) -> bool {
        match self {
            Known(_) => { true }
            Unknown => { false }
        }
    }

    fn is_unknown(&self) -> bool {
        !self.is_known()
    }
}

struct CodeValidatorState {
    operand_stack: Vec<Operand>,
    control_stack: Vec<ControlFrame>,
}

impl CodeValidatorState {
    fn new() -> CodeValidatorState {
        CodeValidatorState {
            operand_stack: Vec::new(),
            control_stack: vec![ControlFrame {
                label_types:Vec::new(),
                end_types: Vec::new(),
                height: 0,
                unreachable: false
            }]
        }
    }

    fn push_known(&mut self, operand: ValueType) {
        self.push_operand(Known(operand));
    }

    fn push_operand(&mut self, operand: Operand) {
        self.operand_stack.push(operand);
    }

    fn pop_operand(&mut self) -> Result<Operand> {
        //TODO:ensure that unwrap doesn't panic
        let last = self.control_stack.last().unwrap();
        if self.operand_stack.len() == last.height as usize {
            if last.unreachable {
                Ok(Unknown)
            } else {
                Err(TypeMismatch)
            }
        } else {
            Ok(self.operand_stack.pop().unwrap())
        }
    }

    fn pop_known(&mut self, expected: ValueType) -> Result<Operand> {
        self.pop_expected(Known(expected))
    }

    fn pop_unknown(&mut self) -> Result<Operand> {
        self.pop_expected(Unknown)
    }

    fn pop_expected(&mut self, expected: Operand) -> Result<Operand> {
        let actual = self.pop_operand()?;
        if actual.is_unknown() {
            return Ok(expected);
        }

        if expected.is_unknown() {
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
        let height = self.operand_stack.len();
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

    fn get_local(locals: &[ValueType], local_index: LocalIndex) -> Result<&ValueType> {
        if let Some(local_type) = locals.get(local_index.0 as usize) {
            Ok(local_type)
        } else {
            Err(InvalidLocalIndex(local_index))
        }
    }

    fn get_global(globals: &[GlobalType], global_index: GlobalIndex) -> Result<&GlobalType> {
        if let Some(global_type) = globals.get(global_index.0 as usize) {
            Ok(global_type)
        } else {
            Err(InvalidGlobalIndex(global_index))
        }
    }

    fn unreachable(&mut self) {
        let last = self.control_stack.last_mut().unwrap();
        self.operand_stack.truncate(last.height);
        last.unreachable = true;
    }

    fn validate_instruction(&mut self,
                            instruction: &Instruction,
                            globals: &[GlobalType],
                            locals: &[ValueType],
    ) -> Result<()> {
        match instruction {
            Instruction::Unreachable => {
                self.unreachable();
            }
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
            Instruction::Drop => {
                self.pop_operand()?;
            }
            Instruction::Select => {
                self.pop_known(ValueType::I32)?;
                let first = self.pop_operand()?;
                let second = self.pop_operand()?;
                if first.is_unknown() || second.is_unknown() || first != second {
                    return Err(TypeMismatch);
                }
                self.push_operand(second);
            }
            Instruction::LocalGet { local_index } => {
                let local_type = Self::get_local(locals, *local_index)?;
                self.push_known(*local_type);
            }
            Instruction::LocalSet { local_index } => {
                let local_type = Self::get_local(locals, *local_index)?;
                self.pop_known(*local_type)?;
            }
            Instruction::LocalTee { local_index } => {
                //TODO: write a generic Vec<IndexType> that accepts an IndexType index
                //and use that everywhere we use Vec<XType>
                let local_type = Self::get_local(locals, *local_index)?;
                self.pop_known(*local_type)?;
                self.push_known(*local_type);
            }
            Instruction::GlobalGet { global_index } => {
                let global_type = Self::get_global(globals, *global_index)?;
                self.push_known(global_type.var_type);
            }
            Instruction::GlobalSet { global_index } => {
                let global_type = Self::get_global(globals, *global_index)?;
                self.pop_known(global_type.var_type)?;
                if !global_type.mutable {
                    return Err(SettingImmutableGlobal(*global_index));
                }
            }
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
            Instruction::I32Const(_) => {
                self.push_known(ValueType::I32);
            }
            Instruction::I64Const(_) => {
                self.push_known(ValueType::I64);
            }
            Instruction::F32Const(_) => {
                self.push_known(ValueType::F32);
            }
            Instruction::F64Const(_) => {
                self.push_known(ValueType::F64);
            }
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

            Instruction::I32Add |
            Instruction::I32Sub |
            Instruction::I32Mul |
            Instruction::I32Divs |
            Instruction::I32Divu |
            Instruction::I32Rems |
            Instruction::I32Remu |
            Instruction::I32And |
            Instruction::I32Or |
            Instruction::I32Xor |
            Instruction::I32Shl |
            Instruction::I32Shrs |
            Instruction::I32Shru |
            Instruction::I32Rotl |
            Instruction::I32Rotr => {
                self.pop_known(ValueType::I32)?;
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }

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