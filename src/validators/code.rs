use crate::{InstructionReader, Instruction, InstructionReaderError, CodeReaderError, BranchReaderError};
use crate::types::{ValueType, GlobalType, GlobalIndex, LocalIndex, TypeIndex, FuncIndex, Locals, FunctionType, MemoryIndex, MemoryArgument, TableIndex, BlockType, LabelIndex, Choice};
use crate::validators::code::CodeValidationError::{InvalidInitExpr, TypeMismatch, InvalidGlobalIndex, InvalidLocalIndex, InvalidTypeIndex, InvalidFunctionIndex, SettingImmutableGlobal, UndefinedMemory, InvalidMemoryAlignment, OperandStackEmpty, UndefinedTable, ValuesAtEndOfBlock, InvalidLabelIndex, TargetLabelsTypeMismatch};
use std::result;
use crate::readers::section::code::{Code, LocalsReader, LocalsIterationProof};
use crate::validators::code::Operand::{Unknown, Known};

#[derive(PartialEq, Eq, Debug)]
pub enum CodeValidationError {
    CodeReader(CodeReaderError),
    InstructionReader(InstructionReaderError),
    BranchReader(BranchReaderError),
    InvalidInitExpr,
    InvalidGlobalIndex(GlobalIndex),
    SettingImmutableGlobal(GlobalIndex),
    InvalidLocalIndex(LocalIndex),
    InvalidTypeIndex(TypeIndex),
    InvalidFunctionIndex(FuncIndex),
    InvalidLabelIndex(LabelIndex),
    UndefinedMemory,
    UndefinedTable,
    InvalidMemoryAlignment,
    TypeMismatch { expected: Operand, actual: Operand },
    TargetLabelsTypeMismatch,
    ValuesAtEndOfBlock,
    OperandStackEmpty,
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

impl From<BranchReaderError> for CodeValidationError {
    fn from(e: BranchReaderError) -> Self {
        CodeValidationError::BranchReader(e)
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
    kind: ControlFrameKind,
    block_type: BlockType,
    height: usize,
    unreachable: bool,
}

fn get_func_type_index(
    function_type_indices: &[TypeIndex],
    function_index: FuncIndex
) -> Result<TypeIndex> {
    Ok(if let Some(func_type_index) = function_type_indices.get(function_index.0 as usize) {
        *func_type_index
    } else {
        return Err(InvalidFunctionIndex(function_index));
    })
}

fn get_func_type<'a>(
    function_types: &'a [FunctionType],
    function_type_indices: &[TypeIndex],
    function_index: FuncIndex
) -> Result<&'a FunctionType> {
    let func_type_index = get_func_type_index(function_type_indices, function_index)?;
    Ok(if let Some(function_type) = function_types.get(func_type_index.0 as usize) {
        function_type
    } else {
        return Err(InvalidTypeIndex(func_type_index));
    })
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
                    function_index: FuncIndex,
                    max_table_index: Option<TableIndex>,
                    max_memory_index: Option<MemoryIndex>,
    ) -> Result<()> {
        let func_type_index = get_func_type_index(function_type_indices, function_index)?;
        let mut state = CodeValidatorState::new(func_type_index);
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
            state.validate_instruction(&instruction, globals, &locals, function_types, function_type_indices, max_table_index, max_memory_index)?;
        }
        Ok(())
    }

    fn create_locals(&self,
                     mut locals_reader: LocalsReader,
                     function_types: &'a [FunctionType],
                     function_type_indices: &[TypeIndex],
                     function_index: FuncIndex
    ) -> Result<(Vec<ValueType>, LocalsIterationProof)> {
        //TODO:For now we are creating a vec of locals,
        //this can be represented more compactly which allows binary search
        //over that compressed representation. Use that representation.
        let mut locals = Vec::new();
        let function_type = get_func_type(function_types, function_type_indices, function_index)?;
        let params = &function_type.params;
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

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    Known(ValueType),
    Unknown,
}

impl Operand {
    fn is_known(&self) -> bool {
        let result = match self {
            Known(_) => { true }
            Unknown => { false }
        };
        result
    }

    fn is_unknown(&self) -> bool {
        let result = !self.is_known();
        result
    }
}

#[derive(PartialEq, Copy, Clone)]
enum ControlFrameKind {
    Block,
    If,
    Else,
    Loop,
}

struct CodeValidatorState {
    operand_stack: Vec<Operand>,
    control_stack: Vec<ControlFrame>,
}

impl CodeValidatorState {
    fn new(type_index: TypeIndex) -> CodeValidatorState {
        CodeValidatorState {
            operand_stack: Vec::new(),
            control_stack: vec![ControlFrame {
                kind: ControlFrameKind::Block,
                block_type: BlockType::TypeIndex(type_index),
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
                Err(OperandStackEmpty)
            }
        } else {
            Ok(self.operand_stack.pop().unwrap())
        }
    }

    fn pop_known(&mut self, expected: ValueType) -> Result<Operand> {
        self.pop_expected(Known(expected))
    }

    // fn pop_unknown(&mut self) -> Result<Operand> {
    //     self.pop_expected(Unknown)
    // }
    //
    fn pop_expected(&mut self, expected: Operand) -> Result<Operand> {
        let actual = self.pop_operand()?;
        if actual.is_unknown() {
            return Ok(expected);
        }

        if expected.is_unknown() {
            return Ok(actual);
        }

        if actual != expected {
            return Err(TypeMismatch { expected, actual });
        }

        Ok(actual)
    }

    // fn pop_operands(&mut self, expected_types: &Vec<ValueType>) -> Result<()> {
    //     for expected in expected_types.iter().rev() {
    //         self.pop_expected_operand(Some(*expected))?;
    //     }
    //     Ok(())
    // }

    fn push_control_frame(&mut self, kind: ControlFrameKind, block_type: BlockType) {
        let height = self.operand_stack.len();
        let frame = ControlFrame { kind, block_type, height, unreachable: false };
        self.control_stack.push(frame);
    }

    fn pop_control_frame(&mut self, function_types: &[FunctionType]) -> Result<ControlFrame> {
        let last = self.control_stack.last().unwrap();
        let height = last.height;

        match last.block_type {
            BlockType::Empty => {}
            BlockType::ValueType(ty) => {
                self.pop_known(ty)?;
            }
            BlockType::TypeIndex(type_index) => {
                let ty = if let Some(function_type) = function_types.get(type_index.0 as usize) {
                    function_type
                } else {
                    return Err(InvalidTypeIndex(type_index));
                };
                for param in ty.results.into_iter().rev() {
                    self.pop_known(*param)?;
                }
            }
        }

        if self.operand_stack.len() != height as usize {
            return Err(ValuesAtEndOfBlock);
        }

        Ok(self.control_stack.pop().unwrap())
    }

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
        //TODO:Ensure the unwrap doesn't panic
        let last = self.control_stack.last_mut().unwrap();
        self.operand_stack.truncate(last.height);
        last.unreachable = true;
    }

    fn validate_table_index(max_table_index: Option<TableIndex>) -> Result<()> {
        if max_table_index.is_none() {
            return Err(UndefinedTable);
        }

        Ok(())
    }

    fn validate_memory_index(max_memory_index: Option<MemoryIndex>) -> Result<()> {
        if max_memory_index.is_none() {
            return Err(UndefinedMemory);
        }

        Ok(())
    }

    fn validate_memory_index_and_alignment(
        max_memory_index: Option<MemoryIndex>,
        memory_argument: &MemoryArgument,
        max_alignment: u32,
    ) -> Result<()> {
        Self::validate_memory_index(max_memory_index)?;
        if memory_argument.alignment > max_alignment {
            return Err(InvalidMemoryAlignment);
        }

        Ok(())
    }

    fn validate_load(&mut self,
                     max_memory_index: Option<MemoryIndex>,
                     memory_argument: &MemoryArgument,
                     max_alignment: u32,
                     result_type: ValueType,
    ) -> Result<()> {
        Self::validate_memory_index_and_alignment(max_memory_index, memory_argument, max_alignment)?;
        self.pop_known(ValueType::I32)?;
        self.push_known(result_type);
        Ok(())
    }

    fn validate_store(&mut self,
                      max_memory_index: Option<MemoryIndex>,
                      memory_argument: &MemoryArgument,
                      max_alignment: u32,
                      param_type: ValueType,
    ) -> Result<()> {
        Self::validate_memory_index_and_alignment(max_memory_index, memory_argument, max_alignment)?;
        self.pop_known(param_type)?;
        self.pop_known(ValueType::I32)?;
        Ok(())
    }

    fn validate_function_type(&mut self, ty: &FunctionType) -> Result<()> {
        for param in ty.params.into_iter().rev() {
            self.pop_known(*param)?;
        }
        for result in ty.results.into_iter() {
            self.push_known(*result);
        }

        Ok(())
    }

    fn validate_block_type(&mut self, kind: ControlFrameKind, block_type: BlockType, function_types: &[FunctionType]) -> Result<()> {
        for ty in block_type.params(function_types)? {
            self.pop_known(ty)?;
        }
        self.push_control_frame(kind, block_type);
        Ok(())
    }

    fn validate_jump(&mut self, label_index: LabelIndex) -> Result<(ControlFrameKind, BlockType)> {
        return match (self.control_stack.len() - 1).checked_sub(label_index.0 as usize) {
            None => {
                Err(InvalidLabelIndex(label_index))
            }
            Some(i) => {
                let frame = &self.control_stack[i];
                Ok((frame.kind, frame.block_type))
            }
        }
    }

    fn get_label_types<'a>(&mut self, kind: ControlFrameKind, block_type: BlockType, function_types: &'a [FunctionType]) -> Result<impl DoubleEndedIterator<Item = ValueType> + 'a> {
        Ok(match kind {
            ControlFrameKind::Loop => { Choice::EitherA(block_type.params(function_types)?) }
            _ => { Choice::OrB(block_type.results(function_types)?) }
        })
    }

    fn validate_instruction(&mut self,
                            instruction: &Instruction,
                            globals: &[GlobalType],
                            locals: &[ValueType],
                            function_types: &[FunctionType],
                            function_type_indices: &[TypeIndex],
                            max_table_index: Option<TableIndex>,
                            max_memory_index: Option<MemoryIndex>,
    ) -> Result<()> {
        match instruction {
            Instruction::Unreachable => {
                self.unreachable();
            }
            Instruction::Nop => {}
            Instruction::Block { block_type } => {
                self.validate_block_type(ControlFrameKind::Block, *block_type, function_types)?;
            }
            Instruction::Loop { block_type } => {
                self.validate_block_type(ControlFrameKind::Loop, *block_type, function_types)?;
            }
            Instruction::If { block_type } => {
                self.pop_known(ValueType::I32)?;
                self.validate_block_type(ControlFrameKind::If, *block_type, function_types)?;
            }
            Instruction::Else => {
                let frame = self.pop_control_frame(function_types)?;
                self.push_control_frame(ControlFrameKind::Else, frame.block_type);
            }
            Instruction::End => {}
            Instruction::Branch { label_index } => {
                let (kind, block_type) = self.validate_jump(*label_index)?;
                for ty in self.get_label_types(kind, block_type, function_types)?.rev() {
                    self.pop_known(ty)?;
                }
                self.unreachable();
            }
            Instruction::BranchIf { label_index } => {
                let (kind, block_type) = self.validate_jump(*label_index)?;
                for ty in self.get_label_types(kind, block_type, function_types)?.rev() {
                    self.pop_known(ty)?;
                }
                for ty in self.get_label_types(kind, block_type, function_types)? {
                    self.push_known(ty);
                }
                self.unreachable();
            }
            Instruction::BranchTable { branch_table_reader } => {
                self.pop_known(ValueType::I32)?;
                let mut reader = branch_table_reader.clone();
                let mut label = None;
                for label_index in reader.into_iter() {
                    let label_index = label_index?;
                    let block = self.validate_jump(label_index)?;
                    match label {
                        None => label = Some(block),
                        Some(prev) => {
                            let a = self.get_label_types(block.0, block.1, function_types)?;
                            let b = self.get_label_types(prev.0, prev.1, function_types)?;
                            if a.ne(b) {
                                return Err(TargetLabelsTypeMismatch);
                            }
                        }
                    }
                }
                let (kind, block_type) = label.unwrap();
                for ty in self.get_label_types(kind, block_type, function_types)?.rev() {
                    self.pop_known(ty)?;
                }
                self.unreachable();
            }
            Instruction::Return => {
                //TODO:Get rid of the clone
                match self.control_stack[0].block_type {
                    BlockType::Empty => {}
                    BlockType::ValueType(ty) => {
                        self.pop_known(ty)?;
                    }
                    BlockType::TypeIndex(type_index) => {
                        if let Some(ty) = function_types.get(type_index.0 as usize) {
                            for param in ty.results.into_iter().rev() {
                                self.pop_known(*param)?;
                            }
                        } else {
                            return Err(InvalidTypeIndex(type_index));
                        }
                    }
                }
                self.unreachable();
            }
            Instruction::Call { func_index } => {
                let ty = get_func_type(function_types, function_type_indices, *func_index)?;
                self.validate_function_type(ty)?;
            }
            Instruction::CallIndirect { type_index } => {
                Self::validate_table_index(max_table_index)?;
                if let Some(ty) = function_types.get(type_index.0 as usize) {
                    self.pop_known(ValueType::I32)?;
                    self.validate_function_type(ty)?;
                } else {
                    return Err(InvalidTypeIndex(*type_index));
                }
            }
            Instruction::Drop => {
                self.pop_operand()?;
            }
            Instruction::Select => {
                self.pop_known(ValueType::I32)?;
                let first = self.pop_operand()?;
                let second = self.pop_operand()?;
                //TODO:should unknown operands be allowed here?
                if first.is_unknown() || second.is_unknown() || first != second {
                    return Err(TypeMismatch { expected: first, actual: second });
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
            Instruction::I32Load { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 2, ValueType::I32)?;
            }
            Instruction::I64Load { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 3, ValueType::I64)?;
            }
            Instruction::F32Load { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 2, ValueType::F32)?;
            }
            Instruction::F64Load { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 3, ValueType::F64)?;
            }
            Instruction::I32Load8s { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 0, ValueType::I32)?;
            }
            Instruction::I32Load8u { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 0, ValueType::I32)?;
            }
            Instruction::I32Load16s { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 1, ValueType::I32)?;
            }
            Instruction::I32Load16u { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 1, ValueType::I32)?;
            }
            Instruction::I64Load8s { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 0, ValueType::I64)?;
            }
            Instruction::I64Load8u { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 0, ValueType::I64)?;
            }
            Instruction::I64Load16s { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 1, ValueType::I64)?;
            }
            Instruction::I64Load16u { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 1, ValueType::I64)?;
            }
            Instruction::I64Load32s { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 2, ValueType::I64)?;
            }
            Instruction::I64Load32u { memory_argument } => {
                self.validate_load(max_memory_index, memory_argument, 2, ValueType::I64)?;
            }
            Instruction::I32Store { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 2, ValueType::I32)?;
            }
            Instruction::I64Store { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 3, ValueType::I64)?;
            }
            Instruction::F32Store { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 2, ValueType::F32)?;
            }
            Instruction::F64Store { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 3, ValueType::F64)?;
            }
            Instruction::I32Store8 { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 0, ValueType::I32)?;
            }
            Instruction::I32Store16 { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 1, ValueType::I32)?;
            }
            Instruction::I64Store8 { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 0, ValueType::I64)?;
            }
            Instruction::I64Store16 { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 1, ValueType::I64)?;
            }
            Instruction::I64Store32 { memory_argument } => {
                self.validate_store(max_memory_index, memory_argument, 2, ValueType::I64)?;
            }
            Instruction::MemorySize => {
                Self::validate_memory_index(max_memory_index)?;
                self.push_known(ValueType::I32);
            }
            Instruction::MemoryGrow => {
                Self::validate_memory_index(max_memory_index)?;
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }
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
            Instruction::I32Eqz => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I32Eq |
            Instruction::I32Ne |
            Instruction::I32Lts |
            Instruction::I32Ltu |
            Instruction::I32Gts |
            Instruction::I32Gtu |
            Instruction::I32Les |
            Instruction::I32Leu |
            Instruction::I32Ges |
            Instruction::I32Geu => {
                self.pop_known(ValueType::I32)?;
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64Eqz => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64Eq |
            Instruction::I64Ne |
            Instruction::I64Lts |
            Instruction::I64Ltu |
            Instruction::I64Gts |
            Instruction::I64Gtu |
            Instruction::I64Les |
            Instruction::I64Leu |
            Instruction::I64Ges |
            Instruction::I64Geu => {
                self.pop_known(ValueType::I64)?;
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::F32Eq |
            Instruction::F32Ne |
            Instruction::F32Lt |
            Instruction::F32Gt |
            Instruction::F32Le |
            Instruction::F32Ge => {
                self.pop_known(ValueType::F32)?;
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::F64Eq |
            Instruction::F64Ne |
            Instruction::F64Lt |
            Instruction::F64Gt |
            Instruction::F64Le |
            Instruction::F64Ge => {
                self.pop_known(ValueType::F64)?;
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I32Clz |
            Instruction::I32Ctz |
            Instruction::I32Popcnt => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }
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
            Instruction::I64Clz |
            Instruction::I64Ctz |
            Instruction::I64Popcnt => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I64);
            }
            Instruction::I64Add |
            Instruction::I64Sub |
            Instruction::I64Mul |
            Instruction::I64Divs |
            Instruction::I64Divu |
            Instruction::I64Rems |
            Instruction::I64Remu |
            Instruction::I64And |
            Instruction::I64Or |
            Instruction::I64Xor |
            Instruction::I64Shl |
            Instruction::I64Shrs |
            Instruction::I64Shru |
            Instruction::I64Rotl |
            Instruction::I64Rotr => {
                self.pop_known(ValueType::I64)?;
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I64);
            }
            Instruction::F32Abs |
            Instruction::F32Neg |
            Instruction::F32Ceil |
            Instruction::F32Floor |
            Instruction::F32Trunc |
            Instruction::F32Nearest |
            Instruction::F32Sqrt => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F32Add |
            Instruction::F32Sub |
            Instruction::F32Mul |
            Instruction::F32Div |
            Instruction::F32Min |
            Instruction::F32Max |
            Instruction::F32Copysign => {
                self.pop_known(ValueType::F32)?;
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F64Abs |
            Instruction::F64Neg |
            Instruction::F64Ceil |
            Instruction::F64Floor |
            Instruction::F64Trunc |
            Instruction::F64Nearest |
            Instruction::F64Sqrt => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::F64);
            }
            Instruction::F64Add |
            Instruction::F64Sub |
            Instruction::F64Mul |
            Instruction::F64Div |
            Instruction::F64Min |
            Instruction::F64Max |
            Instruction::F64Copysign => {
                self.pop_known(ValueType::F64)?;
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::F64);
            }
            Instruction::I32WrapI64 => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I32TruncF32s |
            Instruction::I32TruncF32u => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I32TruncF64s |
            Instruction::I32TruncF64u => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64ExtendI32s |
            Instruction::I64ExtendI32u => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I64);
            }
            Instruction::I64TruncF32s |
            Instruction::I64TruncF32u => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I64);
            }
            Instruction::I64TruncF64s |
            Instruction::I64TruncF64u => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I64);
            }
            Instruction::F32ConvertI32s |
            Instruction::F32ConvertI32u => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F32ConvertI64s |
            Instruction::F32ConvertI64u => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F32DemoteF64 => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F64ConvertI32s |
            Instruction::F64ConvertI32u => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::F64);
            }
            Instruction::F64ConvertI64s |
            Instruction::F64ConvertI64u => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::F64);
            }
            Instruction::F64PromoteF32 => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::F64);
            }
            Instruction::I32ReinterpretF32 => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64ReinterpretF64 => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I64);
            }
            Instruction::F32ReinterpretI32 => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::F32);
            }
            Instruction::F64ReinterpretI64 => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::F64);
            }
            Instruction::I32Extend8s |
            Instruction::I32Extend16s => {
                self.pop_known(ValueType::I32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64Extend8s |
            Instruction::I64Extend16s |
            Instruction::I64Extend32s => {
                self.pop_known(ValueType::I64)?;
                self.push_known(ValueType::I64);
            }
            Instruction::I32TruncSatF32s |
            Instruction::I32TruncSatF32u => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I32TruncSatF64s |
            Instruction::I32TruncSatF64u => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I32);
            }
            Instruction::I64TruncSatF32s |
            Instruction::I64TruncSatF32u => {
                self.pop_known(ValueType::F32)?;
                self.push_known(ValueType::I64);
            }
            Instruction::I64TruncSatF64s |
            Instruction::I64TruncSatF64u => {
                self.pop_known(ValueType::F64)?;
                self.push_known(ValueType::I64);
            }
        }

        Ok(())
    }
}