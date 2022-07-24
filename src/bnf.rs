use std::fmt::Debug;

use crate::parsing_table::ValueStackSymbol;
pub type ReduceArgs<NTV, TV> = Vec<ValueStackSymbol<NTV, TV>>;
pub type ReduceAction<NTV, TV> = Box<dyn Fn(ReduceArgs<NTV, TV>) -> NTV>;

pub struct Grammer<NT, T, NTV, TV>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
    TV: IntoKind<T>,
    NTV: IntoKind<NT>,
{
    pub rules: Vec<Expr<NT, T, NTV, TV>>,
}
pub struct Expr<NT, T, NTV, TV>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
    TV: IntoKind<T>,
    NTV: IntoKind<NT>,
{
    pub left: NT,
    pub right: Vec<Symbol<NT, T>>,
    pub reduce_action: Option<ReduceAction<NTV, TV>>,
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Symbol<NT, T>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
{
    Term(T),
    NonTerm(NT),
}
impl<NT, T> Debug for Symbol<NT, T>
where
    T: Ord + Eq + Clone + Debug,
    NT: Ord + Eq + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Term(arg0) => f.debug_tuple("Term").field(arg0).finish(),
            Self::NonTerm(arg0) => f.debug_tuple("NonTerm").field(arg0).finish(),
        }
    }
}

pub trait EOFSupply<T> {
    fn eof() -> T;
}

pub trait IntoKind<T> {
    fn into_kind(&self) -> T;
}

impl<T> IntoKind<T> for T
where
    T: Clone,
{
    fn into_kind(&self) -> T {
        self.clone()
    }
}
