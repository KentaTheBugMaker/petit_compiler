use std::collections::BTreeSet;

use crate::bnf::Grammer;

pub fn generate_null_set<NT, T>(bnf: &Grammer<NT, T>) -> BTreeSet<NT>
where
    T: Ord + Eq + Clone,
    NT: Ord + Eq + Clone,
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
                            crate::bnf::Alphabet::Term(_) => {
                                //終端記号が含まれるのでヌルになりえない
                                //従ってfalse
                                false
                            }
                            crate::bnf::Alphabet::NonTerm(nt) => {
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
    use crate::bnf::{Alphabet, Expr, Grammer};
    use std::collections::BTreeSet;

    #[derive(Debug, PartialOrd, Ord, Clone, PartialEq, Eq)]
    enum NonTerm {
        X,
        Y,
    }

    #[test]
    fn test_generate_null_set() {
        let grammer = Grammer {
            rules: vec![
                Expr {
                    left: NonTerm::X,
                    right: vec![Alphabet::NonTerm(NonTerm::Y), Alphabet::Term(0)],
                },
                Expr {
                    left: NonTerm::Y,
                    right: vec![Alphabet::Term(1)],
                },
                Expr {
                    left: NonTerm::Y,
                    right: vec![],
                },
            ],
        };
        let nullset = generate_null_set(&grammer);
        let mut ref_set = BTreeSet::new();
        ref_set.insert(NonTerm::Y);
        assert_eq!(nullset, ref_set)
    }
}
