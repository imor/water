#[derive(Eq, PartialEq, Debug)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FuncType {
    pub(crate) params: Box<[ValueType]>,
    pub(crate) results: Box<[ValueType]>,
}

#[derive(Debug)]
pub enum ImportDesc {
    Func { type_index: u32},
    Table(TableType),
    Memory(MemoryType),
    Global,
}

#[derive(Debug)]
pub struct Import<'a> {
    pub(crate) module_name: &'a str,
    pub(crate) name: &'a str,
    pub(crate) import_desc: ImportDesc
}

#[derive(Debug)]
pub struct Limits {
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

#[derive(Debug)]
pub struct TableType {
    pub(crate) limits: Limits
}

#[derive(Debug)]
pub struct MemoryType {
    pub(crate) limits: Limits
}
