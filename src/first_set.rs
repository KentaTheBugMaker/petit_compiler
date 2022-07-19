use crate::bnf::{Grammer, Symbol};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
pub fn generate_first_set<NT, T>(grammer: &Grammer<NT, T>) -> BTreeMap<Symbol<NT, T>, BTreeSet<T>>
where
    T: Ord + Eq + Clone + Debug,
    NT: Ord + Eq + Clone + Debug,
{
    //First集合
    let mut first_sets = BTreeMap::new();
    //すべてのSymbol
    let mut symbols = BTreeSet::new();
    for rule in &grammer.rules {
        symbols.insert(Symbol::NonTerm(rule.left.clone()));
        rule.right.iter().for_each(|symbol| {
            symbols.insert(symbol.clone());
        });
    }
    //First集合の初期化
    for a in &symbols {
        let initial_set = match a {
            Symbol::NonTerm(_) => BTreeSet::new(),
            Symbol::Term(x) => std::iter::once(x).cloned().collect(),
        };
        first_sets.insert(a.clone(), initial_set);
    }
    //ヌル集合
    let nullable_set = crate::nullable_set::generate_null_set(grammer);
    let mut changed = true;
    while changed {
        //包含関係を示す.
        let mut constraints = vec![];

        for rule in &grammer.rules {
            let sup = &rule.left;
            let first_non_null = rule.right.iter().find(|symbol| {
                match symbol {
                    //終端記号なのでヌルになることはない
                    Symbol::Term(_) => false,
                    //ヌル集合にないならばこれ目当てのものになる.
                    Symbol::NonTerm(nt) => nullable_set.contains(nt),
                }
            });

            if let Some(sub) = first_non_null {
                if Symbol::NonTerm(sup.clone()) != sub.clone() {
                    println!("{:?} <-{:?}", sup, sub);
                    constraints.push((sup, sub));
                }
            }
        }

        if constraints.is_empty() {
            changed = false;
        } else {
            //解決.
            println!("solving");
            let snap_shot = first_sets.clone();
            for constraint in constraints {
                let sub = first_sets.get(constraint.1).cloned();

                if let Some(sub) = sub {
                    let super_ = first_sets
                        .get_mut(&Symbol::NonTerm(constraint.0.clone()))
                        .unwrap();
                    sub.iter().for_each(|symbol| {
                        super_.insert(symbol.clone());
                    });
                }
            }
            changed = snap_shot != first_sets;
        }
    }

    first_sets
}

#[cfg(test)]
mod test {
    use crate::bnf::{Expr, Grammer, Symbol};
    use NonTerm::{E, S, T};
    use Symbol::NonTerm as NT;
    use Symbol::Term;

    use super::generate_first_set;
    #[test]
    fn test_generate_first_set() {
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
        let first_set = generate_first_set(&grammer);
        println!("{:#?}", first_set)
    }

    #[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Debug)]
    enum NonTerm {
        S,
        E,
        T,
    }
}
