use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use crate::bnf::{Alphabet, Grammer};

#[derive(Debug)]
pub struct ItemClosure0<NT, T>(pub BTreeSet<LR0Item<NT, T>>)
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug;
pub struct ItemClosure1<NT, T>(pub BTreeSet<LR1Item<NT, T>>)
where
    NT: Ord + Eq + Clone,
    T: Ord + Eq + Clone;

#[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Debug)]
pub struct LR0Item<NT, T>
where
    NT: Ord + Clone + Eq + Debug,
    T: Ord + Eq + Clone + Debug,
{
    pub left: NT,
    pub right: Vec<DotAndAlphabet<NT, T>>,
}

impl<NT, T> std::fmt::Display for LR0Item<NT, T>
where
    T: Ord + Eq + Clone + Debug,
    NT: Ord + Eq + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> ", self.left).ok();
        for dot_and_alphabet in &self.right {
            write!(f, "{}", dot_and_alphabet).ok();
        }
        write!(f, "")
    }
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Clone)]
pub struct LR1Item<NT, T>
where
    NT: Ord + Clone + Eq,
    T: Ord + Eq + Clone,
{
    pub left: NT,
    pub right: Vec<DotAndAlphabet<NT, T>>,
    pub lookahead: T,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Debug)]
pub enum DotAndAlphabet<NT, T>
where
    NT: Ord + Clone + Eq,
    T: Ord + Clone + Eq,
{
    Dot,
    Symbol(Alphabet<NT, T>),
}

impl<NT, T> std::fmt::Display for DotAndAlphabet<NT, T>
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dot => write!(f, "・"),
            Self::Symbol(arg0) => match arg0 {
                Alphabet::Term(t) => write!(f, "{:?}", t),
                Alphabet::NonTerm(nt) => write!(f, "{:?}", nt),
            },
        }
    }
}

impl<NT, T> Into<LR1Item<NT, T>> for (LR0Item<NT, T>, T)
where
    NT: Clone + Eq + Ord + Debug,
    T: Clone + Eq + Ord + Debug,
{
    fn into(self) -> LR1Item<NT, T> {
        let (lr0, la) = self;
        LR1Item {
            left: lr0.left,
            right: lr0.right,
            lookahead: la,
        }
    }
}

///情報系教科書シリーズ　コンパイラ　によれば lr0項とは次のようなものである.
/// 生成規則の右辺に1つだけ .をつけたものをいう.
/// * E -> .E+T 導入項
/// * E -> E.+T
/// * E -> E+.T
/// * R -> E+T. 完全項
pub fn generate_lr0_item_set<NT, T>(grammer: &Grammer<NT, T>) -> Vec<LR0Item<NT, T>>
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
{
    let mut set = Vec::new();

    //ルールごとにLR0Itemを作っていく.
    for rule in &grammer.rules {
        //ドット挿入位置
        let positions = 0..=rule.right.len();
        let item: Vec<DotAndAlphabet<NT, T>> = rule
            .right
            .iter()
            .map(|alphabet| DotAndAlphabet::Symbol(alphabet.clone()))
            .collect();
        for position in positions {
            let mut item = item.clone();
            item.insert(position, DotAndAlphabet::Dot);
            let lr0_item = LR0Item {
                left: rule.left.clone(),
                right: item,
            };
            set.push(lr0_item);
        }
    }
    set
}

///LR0クロージャを計算する.
///
/// * lr0_items 文法のすべてのLR0項  
/// * i クロージャを作りたい項の集合
///
///情報系教科書シリーズ　コンパイラ　によれば Closureとは次のようにして作ることができる.
///
/// 入力された集合に対して導入項を追加していく.
///
pub fn generate_lr0_item_closure<NT, T>(
    lr0_items: &[LR0Item<NT, T>],
    i: &[LR0Item<NT, T>],
) -> Vec<LR0Item<NT, T>>
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
{
    let mut finished_set = BTreeSet::new();
    let mut finished_nonterms = BTreeSet::new();
    let mut i_dash = i.to_vec();
    let induction_terms: BTreeSet<LR0Item<NT, T>> = lr0_items
        .iter()
        .filter(|lr0_item| {
            if let Some(DotAndAlphabet::Dot) = lr0_item.right.get(0) {
                true
            } else {
                false
            }
        })
        .cloned()
        .collect();
    let mut changed = true;
    while changed {
        let ss = i_dash.clone();
        let mut requires = BTreeSet::new();
        for item in i_dash.iter() {
            if !finished_set.contains(item) {
                //まずは.の後ろが非終端記号か判定する.
                let mut finder =
                    item.right
                        .iter()
                        .skip_while(|dot_or_alphabet| match dot_or_alphabet {
                            DotAndAlphabet::Dot => false,
                            DotAndAlphabet::Symbol(_) => true,
                        });
                finder.next();

                //非終端記号なので要求リストに入れる.
                if let Some(DotAndAlphabet::Symbol(Alphabet::NonTerm(nt))) = finder.next() {
                    //すでに要求し終わっているならば入れない
                    if !finished_nonterms.contains(nt) {
                        requires.insert(nt.clone());
                        //要求リストに入れたらこの項での仕事は終わり.
                        finished_set.insert(item.clone());
                        finished_nonterms.insert(nt.clone());
                    }
                }
            }
        }
        //追加していく
        for require in requires {
            println!("required {:?}", require);
            let induction_terms = induction_terms
                .iter()
                .filter(|lr0_item| lr0_item.left == require)
                .cloned();
            i_dash.extend(induction_terms);
        }
        println!("added items");
        changed = ss != i_dash;
    }
    i_dash
}

///情報系教科書シリーズ　コンパイラ　によれば Goto(Itemset,a)とは次のようにして作ることができる.
///
///ドットの直後にaがあるものを集めてドット位置を右に一つずらしたもののクロージャをとる.
pub fn generate_goto_set<NT, T>(
    grammer: &Grammer<NT, T>,
    lr0_set: &[LR0Item<NT, T>],
    alphabet: Alphabet<NT, T>,
) -> Vec<LR0Item<NT, T>>
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
{
    let full_set = generate_lr0_item_set(grammer);
    //ドットの直後に alphabetがあるものを集めて.
    let target_items = lr0_set.iter().filter(|item| {
        let mut finder = item
            .right
            .iter()
            .skip_while(|dot_or_alphabet| match dot_or_alphabet {
                DotAndAlphabet::Dot => false,
                DotAndAlphabet::Symbol(_) => true,
            });
        finder.next();
        if let Some(DotAndAlphabet::Symbol(ap)) = finder.next() {
            ap.clone() == alphabet
        } else {
            false
        }
    });
    //ドットを一つすすめる.
    let i: Vec<LR0Item<NT, T>> = target_items
        .map(|item| {
            let dot_pos = item
                .right
                .iter()
                .enumerate()
                .skip_while(|dot_or_alphabet| match dot_or_alphabet.1 {
                    DotAndAlphabet::Dot => false,
                    _ => true,
                })
                .next()
                .unwrap()
                .0;
            let mut right = item.right.clone();
            right.swap(dot_pos, dot_pos + 1);
            LR0Item {
                left: item.left.clone(),
                right,
            }
        })
        .collect();
    println!("generate for ");
    for item in i.iter() {
        println!("{}", item);
    }
    generate_lr0_item_closure(&full_set, &i)
}

/// 正準オートマトンを作成する.
/// * grammer 文法,
/// * start_symbol 開始記号(左辺のみにあり,OR規則でないこと)
///   ``` S -> S' のような形になっている Sのこと. ```
pub fn generate_canonical_automaton<NT, T>(
    grammer: &Grammer<NT, T>,
    start_symbol: NT,
    alphabets: &[Alphabet<NT, T>],
) -> (
    Vec<Vec<LR0Item<NT, T>>>,
    BTreeMap<(Vec<LR0Item<NT, T>>, Alphabet<NT, T>), Vec<LR0Item<NT, T>>>,
)
where
    NT: Ord + Clone + Eq + Debug,
    T: Ord + Clone + Eq + Debug,
{
    let items = generate_lr0_item_set(grammer);
    let start_rule = items.iter().find(|item| item.left.clone() == start_symbol);
    if let Some(start_rule) = start_rule {
        let ie = generate_lr0_item_closure(&items, &[start_rule.clone()]);
        let mut x = vec![ie];
        let mut y = vec![];
        //状態遷移表
        let mut delta = BTreeMap::new();
        while !x.is_empty() {
            let i = x.remove(0);
            y.push(i.clone());
            for alphabet in alphabets {
                println!("I' = Goto({:?},{:?})", i, alphabet);
                let i_dash = generate_goto_set(grammer, &i, alphabet.clone());
                println!("I' = {:?}", i_dash);
                if !i_dash.is_empty() {
                    if !y.contains(&i_dash) & !x.contains(&i_dash) {
                        x.push(i_dash.clone());
                    }
                    delta.insert((i.clone(), alphabet.clone()), i_dash);
                }
            }
        }
        (y, delta)
    } else {
        (vec![], BTreeMap::new())
    }
}

pub fn compile_canonical_automaton_to_dot<NT, T>(
    automaton: (
        &[Vec<LR0Item<NT, T>>],
        &BTreeMap<(Vec<LR0Item<NT, T>>, Alphabet<NT, T>), Vec<LR0Item<NT, T>>>,
    ),
    automaton_name: &str,
) -> String
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
{
    let node_with_id: BTreeMap<Vec<LR0Item<NT, T>>, usize> = automaton
        .0
        .iter()
        .enumerate()
        .map(|(id, set)| (set.clone(), id))
        .collect();
    println!("nodes = {}", node_with_id.len());
    format!(
        "digraph {} {{
{}}}",
        automaton_name,
        {
            let mut buffer = String::new();
            let shifts = automaton
                .1
                .iter()
                .map(|((from, symbol), to)| (node_with_id.get(from), symbol, node_with_id.get(to)));
            for shift in shifts {
                buffer.push_str(&format!(
                    "I{} -> I{} [{}];\n",
                    shift.0.unwrap(),
                    shift.2.unwrap(),
                    match shift.1 {
                        Alphabet::Term(t) => {
                            format!("label=\"{:?}\", style=solid", t)
                        }
                        Alphabet::NonTerm(nt) => {
                            format!("label=\"{:?}\", style=bold", nt)
                        }
                    },
                ));
            }
            buffer
        }
    )
}

#[cfg(test)]
mod test {
    use super::generate_lr0_item_set;
    use crate::bnf::{Alphabet, Expr, Grammer};
    use crate::item_set::{
        compile_canonical_automaton_to_dot, generate_canonical_automaton, generate_goto_set,
        generate_lr0_item_closure, DotAndAlphabet, LR0Item,
    };
    use Alphabet::NonTerm as NT;
    use Alphabet::Term;
    use NonTerm::{E, S, T};
    #[test]
    fn test_generate_lr0_item_set() {
        let grammer = Grammer {
            rules: vec![
                Expr {
                    left: S,
                    right: vec![NT(E)],
                },
                Expr {
                    left: E,
                    right: vec![NT(T)],
                },
                Expr {
                    left: E,
                    right: vec![Term('('), NT(E), Term(')')],
                },
                Expr {
                    left: T,
                    right: vec![Term('n')],
                },
                Expr {
                    left: T,
                    right: vec![Term('+'), NT(T)],
                },
                Expr {
                    left: T,
                    right: vec![NT(T), Term('+'), Term('n')],
                },
            ],
        };
        let lr0_item_set = generate_lr0_item_set(&grammer);
        for item in lr0_item_set {
            println!("{}", item);
        }
    }

    #[test]
    fn test_generate_lr0_item_closure() {
        #[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Debug)]
        enum NT {
            E,
            T,
            F,
        }
        let grammer = Grammer {
            rules: vec![
                Expr {
                    left: NT::E,
                    right: vec![
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term('+'),
                        Alphabet::NonTerm(NT::T),
                    ],
                },
                Expr {
                    left: NT::E,
                    right: vec![Alphabet::NonTerm(NT::T)],
                },
                Expr {
                    left: NT::T,
                    right: vec![
                        Alphabet::NonTerm(NT::T),
                        Alphabet::Term('*'),
                        Alphabet::NonTerm(NT::F),
                    ],
                },
                Expr {
                    left: NT::T,
                    right: vec![Alphabet::NonTerm(NT::F)],
                },
                Expr {
                    left: NT::F,
                    right: vec![
                        Alphabet::Term('('),
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term(')'),
                    ],
                },
                Expr {
                    left: NT::F,
                    right: vec![Alphabet::Term('i')],
                },
            ],
        };
        let closure_target = LR0Item {
            left: NT::E,
            right: vec![
                DotAndAlphabet::Symbol(Alphabet::NonTerm(NT::E)),
                DotAndAlphabet::Symbol(Alphabet::Term('+')),
                DotAndAlphabet::Dot,
                DotAndAlphabet::Symbol(Alphabet::NonTerm(NT::T)),
            ],
        };
        let lr0_items = generate_lr0_item_set(&grammer);
        let item_closure = generate_lr0_item_closure(&lr0_items, &[closure_target]);
        for item in item_closure {
            println!("{}", item);
        }
    }

    #[test]
    fn test_generate_goto_set() {
        #[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Debug)]
        enum NT {
            E,
            T,
            F,
        }
        let grammer = Grammer {
            rules: vec![
                Expr {
                    left: NT::E,
                    right: vec![
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term('+'),
                        Alphabet::NonTerm(NT::T),
                    ],
                },
                Expr {
                    left: NT::E,
                    right: vec![Alphabet::NonTerm(NT::T)],
                },
                Expr {
                    left: NT::T,
                    right: vec![
                        Alphabet::NonTerm(NT::T),
                        Alphabet::Term('*'),
                        Alphabet::NonTerm(NT::F),
                    ],
                },
                Expr {
                    left: NT::T,
                    right: vec![Alphabet::NonTerm(NT::F)],
                },
                Expr {
                    left: NT::F,
                    right: vec![
                        Alphabet::Term('('),
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term(')'),
                    ],
                },
                Expr {
                    left: NT::F,
                    right: vec![Alphabet::Term('i')],
                },
            ],
        };

        let goto_set = generate_goto_set(
            &grammer,
            &[LR0Item {
                left: NT::T,
                right: vec![
                    DotAndAlphabet::Symbol(Alphabet::NonTerm(NT::T)),
                    DotAndAlphabet::Dot,
                    DotAndAlphabet::Symbol(Alphabet::Term('*')),
                    DotAndAlphabet::Symbol(Alphabet::NonTerm(NT::F)),
                ],
            }],
            Alphabet::Term('*'),
        );
        for item in goto_set {
            println!("{}", item);
        }
    }

    #[test]
    fn test_generate_canonical_set() {
        #[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Debug)]
        enum NT {
            S,
            E,
            T,
            F,
        }
        let grammer = Grammer {
            rules: vec![
                Expr {
                    left: NT::S,
                    right: vec![Alphabet::NonTerm(NT::E)],
                },
                Expr {
                    left: NT::E,
                    right: vec![
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term('+'),
                        Alphabet::NonTerm(NT::T),
                    ],
                },
                Expr {
                    left: NT::E,
                    right: vec![Alphabet::NonTerm(NT::T)],
                },
                Expr {
                    left: NT::T,
                    right: vec![
                        Alphabet::NonTerm(NT::T),
                        Alphabet::Term('*'),
                        Alphabet::NonTerm(NT::F),
                    ],
                },
                Expr {
                    left: NT::T,
                    right: vec![Alphabet::NonTerm(NT::F)],
                },
                Expr {
                    left: NT::F,
                    right: vec![
                        Alphabet::Term('('),
                        Alphabet::NonTerm(NT::E),
                        Alphabet::Term(')'),
                    ],
                },
                Expr {
                    left: NT::F,
                    right: vec![Alphabet::Term('i')],
                },
            ],
        };

        let canonical_automaton = generate_canonical_automaton(
            &grammer,
            NT::S,
            &[
                Alphabet::NonTerm(NT::S),
                Alphabet::NonTerm(NT::E),
                Alphabet::NonTerm(NT::T),
                Alphabet::NonTerm(NT::F),
                Alphabet::Term('i'),
                Alphabet::Term('('),
                Alphabet::Term(')'),
                Alphabet::Term('+'),
                Alphabet::Term('*'),
            ],
        );
        let canonical_set = canonical_automaton.0.clone();
        //compress.
        for items in canonical_set.iter().enumerate() {
            println!("I{}", items.0);
            for item in items.1 {
                println!("{}", item);
            }
        }
        println!(
            "graph:
        {}
        ",
            compile_canonical_automaton_to_dot(
                (&canonical_automaton.0, &canonical_automaton.1),
                "G1"
            )
        )
    }

    #[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Debug)]
    enum NonTerm {
        S,
        E,
        T,
    }
}
