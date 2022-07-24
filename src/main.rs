use bnf::IntoKind;
use item_set::compile_canonical_automaton_to_dot;

use crate::{
    bnf::{Expr, Grammer, ReduceAction, ReduceArgs, Symbol},
    item_set::generate_canonical_automaton,
    parsing_table::{canonical_automaton_to_lr0_parser, ValueStackSymbol},
};

use std::fmt::Debug;

mod bnf;
mod first_set;
mod item_set;
mod nullable_set;
mod parsing_table;
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum NT {
    S,
    Sdash,
    E,
    P,
}

impl Debug for NT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S => write!(f, "S"),
            Self::Sdash => write!(f, "S'"),
            Self::E => write!(f, "E"),
            Self::P => write!(f, "P"),
        }
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum T {
    One,
    Plus,
    LP,
    RP,
    Eof,
}

impl Debug for T {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Plus => write!(f, "+"),
            Self::LP => write!(f, "("),
            Self::RP => write!(f, ")"),
            Self::Eof => write!(f, "$"),
        }
    }
}

//This is AST.
#[derive(Clone, Debug)]
enum S {
    E(E),
}
#[derive(Clone, Debug)]
enum E {
    EPlusP(Box<E>, Box<P>),
    P(Box<P>),
}

#[derive(Clone, Debug)]
enum P {
    Expression(Box<E>),
    One,
}
#[derive(Clone, Debug)]
enum NTV {
    S(S),
    E(E),
    P(P),
}
impl IntoKind<NT> for NTV {
    fn into_kind(&self) -> NT {
        match self {
            NTV::S(_) => NT::S,
            NTV::E(_) => NT::E,
            NTV::P(_) => NT::P,
        }
    }
}

fn main() {
    let grammer = Grammer {
        rules: vec![
            Expr {
                left: NT::Sdash,
                right: vec![Symbol::NonTerm(NT::S), Symbol::Term(T::Eof)],
                reduce_action: None,
            },
            Expr {
                left: NT::S,
                right: vec![
                    Symbol::Term(T::LP),
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::RP),
                ],
                reduce_action: Some(Box::new(|args: ReduceArgs<NTV, T>| -> NTV {
                    if let ValueStackSymbol::NonTerm(NTV::E(e)) = &args[1] {
                        NTV::S(S::E(e.clone()))
                    } else {
                        panic!("")
                    }
                }) as ReduceAction<_, _>),
            },
            Expr {
                left: NT::E,
                right: vec![
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::Plus),
                    Symbol::NonTerm(NT::P),
                ],
                reduce_action: Some(Box::new(|args: ReduceArgs<NTV, T>| -> NTV {
                    let e = &args[0];
                    let p = &args[2];
                    if let (
                        ValueStackSymbol::NonTerm(NTV::E(e)),
                        ValueStackSymbol::NonTerm(NTV::P(p)),
                    ) = (e, p)
                    {
                        NTV::E(E::EPlusP(Box::new(e.clone()), Box::new(p.clone())))
                    } else {
                        panic!("")
                    }
                })),
            },
            Expr {
                left: NT::E,
                right: vec![Symbol::NonTerm(NT::P)],
                reduce_action: Some(Box::new(|args: ReduceArgs<NTV, T>| -> NTV {
                    if let ValueStackSymbol::NonTerm(NTV::P(p)) = &args[0] {
                        NTV::E(E::P(Box::new(p.clone())))
                    } else {
                        panic!("")
                    }
                })),
            },
            Expr {
                left: NT::P,
                right: vec![
                    Symbol::Term(T::LP),
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::RP),
                ],
                reduce_action: Some(Box::new(|args: ReduceArgs<NTV, T>| -> NTV {
                    if let ValueStackSymbol::NonTerm(NTV::E(e)) = &args[1] {
                        NTV::P(P::Expression(Box::new(e.clone())))
                    } else {
                        panic!("")
                    }
                })),
            },
            Expr {
                left: NT::P,
                right: vec![Symbol::Term(T::One)],
                reduce_action: Some(Box::new(|_: ReduceArgs<NTV, T>| -> NTV { NTV::P(P::One) })),
            },
        ],
    };

    let (states, goto, reduce_action) = generate_canonical_automaton(
        grammer,
        NT::Sdash,
        &[
            Symbol::NonTerm(NT::Sdash),
            Symbol::NonTerm(NT::S),
            Symbol::NonTerm(NT::E),
            Symbol::NonTerm(NT::P),
            Symbol::Term(T::One),
            Symbol::Term(T::Plus),
            Symbol::Term(T::LP),
            Symbol::Term(T::RP),
            Symbol::Term(T::Eof),
        ],
    );
    println!(
        "{}",
        compile_canonical_automaton_to_dot((&states, &goto), "")
    );
    let terms = [T::One, T::Plus, T::LP, T::RP, T::Eof];
    let parser = canonical_automaton_to_lr0_parser(
        (&states, &goto, reduce_action),
        NT::Sdash,
        NT::S,
        T::Eof,
        &terms,
    );

    println!();
    let nonterms = [NT::S, NT::E, NT::P];
    parser.export_as_latex_src(&terms, &nonterms);

    /*
     ((1)+(1+1))
    */
    let mut parser = parser.input(vec![
        T::LP,
        T::LP,
        T::One,
        T::RP,
        T::Plus,
        T::LP,
        T::One,
        T::Plus,
        T::One,
        T::RP,
        T::RP,
        T::Eof,
    ]);
    println!();
    parser.export_parsing_as_latex_src();
    println!("{:#?}", parser.get_syntax_tree());
}
