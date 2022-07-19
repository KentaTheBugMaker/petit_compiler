use item_set::compile_canonical_automaton_to_dot;

use crate::{
    bnf::{Expr, Grammer, Symbol},
    item_set::generate_canonical_automaton,
    parsing_table::canonical_automaton_to_lr0_parser,
};

use std::fmt::Debug;

mod bnf;
mod first_set;
mod follow_set;
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
    EOF,
}

impl Debug for T {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Plus => write!(f, "+"),
            Self::LP => write!(f, "("),
            Self::RP => write!(f, ")"),
            Self::EOF => write!(f, "$"),
        }
    }
}

fn main() {
    let grammer = Grammer {
        rules: vec![
            Expr {
                left: NT::Sdash,
                right: vec![Symbol::NonTerm(NT::S), Symbol::Term(T::EOF)],
            },
            Expr {
                left: NT::S,
                right: vec![
                    Symbol::Term(T::LP),
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::RP),
                ],
            },
            Expr {
                left: NT::E,
                right: vec![
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::Plus),
                    Symbol::NonTerm(NT::P),
                ],
            },
            Expr {
                left: NT::E,
                right: vec![Symbol::NonTerm(NT::P)],
            },
            Expr {
                left: NT::P,
                right: vec![
                    Symbol::Term(T::LP),
                    Symbol::NonTerm(NT::E),
                    Symbol::Term(T::RP),
                ],
            },
            Expr {
                left: NT::P,
                right: vec![Symbol::Term(T::One)],
            },
        ],
    };
    let (states, goto) = generate_canonical_automaton(
        &grammer,
        NT::Sdash,
        &vec![
            Symbol::NonTerm(NT::Sdash),
            Symbol::NonTerm(NT::S),
            Symbol::NonTerm(NT::E),
            Symbol::NonTerm(NT::P),
            Symbol::Term(T::One),
            Symbol::Term(T::Plus),
            Symbol::Term(T::LP),
            Symbol::Term(T::RP),
            Symbol::Term(T::EOF),
        ],
    );
    println!(
        "{}",
        compile_canonical_automaton_to_dot((&states, &goto), "")
    );
    let terms = [T::One, T::Plus, T::LP, T::RP, T::EOF];
    let parser =
        canonical_automaton_to_lr0_parser((&states, &goto), NT::Sdash, NT::S, T::EOF, &terms);

    println!("");
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
        T::EOF,
    ]);
    println!("");
    parser.export_parsing_as_latex_src();
}
