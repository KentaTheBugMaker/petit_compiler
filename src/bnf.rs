use std::fmt::Debug;

pub struct Grammer<NT, T>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
{
    pub rules: Vec<Expr<NT, T>>,
}
pub struct Expr<NT, T>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
{
    pub left: NT,
    pub right: Vec<Alphabet<NT, T>>,
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Alphabet<NT, T>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
{
    Term(T),
    NonTerm(NT),
}
impl<NT, T> Debug for Alphabet<NT, T>
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
