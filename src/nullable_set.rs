use std::collections::BTreeSet;

use crate::bnf::{Grammer, IntoKind};

#[allow(dead_code)]
pub fn generate_null_set<NT, T, NTV, TV>(bnf: &Grammer<NT, T, NTV, TV>) -> BTreeSet<NT>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
    NTV: IntoKind<NT>,
    TV: IntoKind<T>,
{
    let mut set = BTreeSet::new();
    for rule in &bnf.rules {
        if rule.right.is_empty() {
            set.insert(rule.left.clone());
        }
    }
    let mut flag = true;
    while flag {
        flag = false;
        for rule in &bnf.rules {
            if set.contains(&rule.left) {
                continue;
            } else {
                let is_null = rule
                    .right
                    .iter()
                    .map(|tk| {
                        match tk {
                            crate::bnf::Symbol::Term(_) => {
                                //終端記号が含まれるのでヌルになりえない
                                //従ってfalse
                                false
                            }
                            crate::bnf::Symbol::NonTerm(nt) => {
                                //ヌル集合に含まれている
                                set.contains(nt)
                            }
                        }
                    })
                    .fold(true, |i, j| i & j);
                if is_null {
                    flag = true;
                    set.insert(rule.left.clone());
                }
            }
        }
    }

    set
}

#[cfg(test)]
mod test {

    use super::generate_null_set;
    use crate::bnf::{Expr, Grammer, Symbol};
    use std::collections::BTreeSet;

    #[derive(Debug, PartialOrd, Ord, Clone, PartialEq, Eq)]
    enum NonTerm {
        X,
        Y,
    }

    #[test]
    fn test_generate_null_set() {
        let grammer: Grammer<NonTerm, i32, NonTerm, i32> = Grammer {
            rules: vec![
                Expr {
                    left: NonTerm::X,
                    right: vec![Symbol::NonTerm(NonTerm::Y), Symbol::Term(0)],
                    reduce_action: None,
                },
                Expr {
                    left: NonTerm::Y,
                    right: vec![Symbol::Term(1)],
                    reduce_action: None,
                },
                Expr {
                    left: NonTerm::Y,
                    right: vec![],
                    reduce_action: None,
                },
            ],
        };
        let nullset = generate_null_set(&grammer);
        let mut ref_set = BTreeSet::new();
        ref_set.insert(NonTerm::Y);
        assert_eq!(nullset, ref_set)
    }
}
