#[derive(Eq, PartialEq, Debug)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FuncType {
    pub(crate) params: Box<[Type]>,
    pub(crate) results: Box<[Type]>,
}