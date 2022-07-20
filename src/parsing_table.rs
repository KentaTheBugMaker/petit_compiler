use crate::{bnf::{Symbol, IntoKind}, item_set::LR0Item};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

#[derive(Debug)]
enum ActionKind {
    Accept,
    Reduce(usize),
    Shift(usize),
    Error,
}

pub struct LR0Parser<NT, T,NTV,TV>
where
    NT: Debug + Clone + Eq + Ord,
    T: Debug + Clone + Eq + Ord,
    NTV:IntoKind<NT>,
    TV:IntoKind<T>,
{
    input: Vec<TV>,
    // (q,a)->p
    action_table: BTreeMap<(usize, T), ActionKind>,
    goto_table: BTreeMap<(usize, NT), usize>,
    stack: Vec<usize>,
    //rules
    rule_table: Vec<LR0Item<NT, T>>,
    // value_stack
    value_stack:Vec<ValueStackSymbol<NTV,TV>>,
    // reduce_action_table.
    reduce_action_table:BTreeMap<LR0Item<NT,T>,Box<dyn Fn(&[ValueStackSymbol<NTV,TV>])->NTV>>,
}

#[derive(Debug)]
pub enum ValueStackSymbol<NTV,TV>{
    NonTerm(NTV),
    Term(TV),
}

/*
    正準オートマトン　から LR(0)構文解析器を作成する.
*/
pub fn canonical_automaton_to_lr0_parser<NT, T,NTV,TV>(
    automaton: (
        &[Vec<LR0Item<NT, T>>],
        &BTreeMap<(Vec<LR0Item<NT, T>>, Symbol<NT, T>), Vec<LR0Item<NT, T>>>,
    ),
    extended_start_symbol: NT,
    start_symbol: NT,
    eof_symbol: T,
    terms: &[T],
) -> LR0Parser<NT, T,NTV,TV>
where
    NT: Ord + Eq + Clone + Debug,
    T: Ord + Eq + Clone + Debug,
    NTV:IntoKind<NT>,
    TV:IntoKind<T>
{
    //状態番号をつける

    let state_number_table: BTreeMap<_, _> = automaton
        .0
        .iter()
        .enumerate()
        .map(|(id, state)| (state.clone(), id))
        .collect();
    let start_rule = LR0Item {
        left: extended_start_symbol.clone(),
        right: vec![
            Symbol::NonTerm(start_symbol.clone()),
            Symbol::Term(eof_symbol.clone()),
        ],
        dot_pos: 0,
    };
    let start_state_number = state_number_table
        .iter()
        .find(|(state, _)| state.contains(&start_rule))
        .unwrap()
        .1;
    // Accept を探すために使う.
    let accept_rule = LR0Item {
        left: extended_start_symbol,
        right: vec![Symbol::NonTerm(start_symbol), Symbol::Term(eof_symbol)],
        dot_pos: 2,
    };
    let mut action_table = BTreeMap::new();
    let mut goto_table = BTreeMap::new();
    //還元を行う状態の集合.
    let reduce_states: BTreeSet<_> = automaton
        .0
        .iter()
        .filter(|state| {
            let reduce_state: BTreeSet<_> = state
                .iter()
                .filter(|lr0item| {
                    lr0item.dot_pos == lr0item.right.len() && ((*lr0item).clone() != accept_rule)
                })
                .collect();
            if reduce_state.len() > 1 {
                eprintln!("Reduce/Reduce conflict detected.");
                true
            } else if reduce_state.len() == 1 && state.len() > 1 {
                println!(" Shift/Reduce conflict detected.");
                true
            } else {
                reduce_state.len() == 1
            }
        })
        .collect();
    let mut rule_table = vec![];
    for (rule_number, reduce_state) in reduce_states.iter().enumerate() {
        if let Some(state_number) = state_number_table.get(*reduce_state) {
            rule_table.push(reduce_state[0].clone());
            for term in terms {
                action_table.insert(
                    (*state_number, term.clone()),
                    ActionKind::Reduce(rule_number),
                );
            }
        }
    }

    for ((from, symbol), to) in automaton.1 {
        match symbol {
            Symbol::Term(t) => {
                let rule = if to.contains(&accept_rule) {
                    ActionKind::Accept
                } else if !to.is_empty() {
                    ActionKind::Shift(*state_number_table.get(to).unwrap())
                } else {
                    ActionKind::Error
                };
                let action_entry = ((*state_number_table.get(from).unwrap(), t.clone()), rule);
                action_table.insert(action_entry.0, action_entry.1);
            }
            Symbol::NonTerm(nt) => {
                goto_table.insert(
                    (*state_number_table.get(from).unwrap(), nt.clone()),
                    *state_number_table.get(to).unwrap(),
                );
            }
        }
    }

    LR0Parser {
        input: vec![],
        action_table,
        goto_table,
        stack: vec![*start_state_number],
        rule_table,
        value_stack: Vec::new(),
        reduce_action_table: BTreeMap::new()
    }
}

impl<NT, T,NTV,TV> LR0Parser<NT, T,NTV,TV>
where
    NT: Clone + Eq + Ord + Debug,
    T: Clone + Eq + Ord + Debug,
    NTV:IntoKind<NT>,
    TV:IntoKind<T>,
{
    pub fn export_as_latex_src(&self, terms: &[T], nonterms: &[NT])
    where
        NT: Ord + Eq + Clone + Debug,
        T: Ord + Eq + Clone + Debug,
    {
        let terms: BTreeSet<_> = terms.iter().collect();
        let nonterms: BTreeSet<_> = nonterms.iter().collect();

        println!(
            "generating LaTeX source file.
        You can insert this generated snippet to table enviroment.
        But may not compile due to escape characters. 
        \n"
        );

        println!(
            "\\begin{{tabular}}{{{}}}",
            "l".repeat(1 + terms.len() + 1 + nonterms.len())
        );
        println!(
            "& \\multicolumn{{{}}}{{c}}{{Action}} & & \\multicolumn{{{}}}{{c}}{{Goto}}  \\\\ \\hline",
            terms.len() ,
            nonterms.len() 
        );
        println!(
            " &{}  &{} \\\\",
            {
                let mut buffer = String::new();
                use std::fmt::Write;
                terms.iter().for_each(|term| {
                    write!(&mut buffer, " {:?} &", term).unwrap();
                });
                buffer
            },
            {
                use std::fmt::Write;
                let mut buffer = String::new();
                let ln = nonterms.len() - 1;
                nonterms.iter().take(ln).for_each(|nonterm| {
                    write!(&mut buffer, "{:?} &", nonterm).unwrap();
                });
                write!(&mut buffer, " {:?}", nonterms.iter().nth(ln).unwrap()).unwrap();

                buffer
            }
        );
        let max_state_number = self.action_table.iter().map(|((n, _), _)| n).max().unwrap();
        for state_number in 0..=*max_state_number {
            let mut row = String::new();
            use std::fmt::Write;
            write!(&mut row, "$ q_{{{}}} $ ", state_number).unwrap();
            //Print action table
            for t in terms.iter() {
                if let Some(action) = self.action_table.get(&(state_number, (*t).clone())) {
                    match action {
                        ActionKind::Accept => row.push_str(" & Accept "),
                        ActionKind::Reduce(rule_number) => {
                            write!(&mut row, "& Reduce( $ r_{{{}}} $ ) ", rule_number).unwrap();
                        }
                        ActionKind::Shift(next_state) => {
                            write!(&mut row, "& Shift( $ q_{{{}}} $ ) ", next_state).unwrap();
                        }
                        ActionKind::Error => {
                            write!(&mut row, "& error ").unwrap();
                        }
                    }
                } else {
                    row.push('&')
                }
            }
            row.push('&');
            //Print goto table
            for nt in nonterms.iter() {
                if let Some(state) = self.goto_table.get(&(state_number, (*nt).clone())) {
                    write!(&mut row, "& $ q_{{{}}} $ ", state).unwrap();
                } else {
                    row.push('&')
                }
            }
            row.push_str(r"\\\hline");
            println!("{}", row);
        }
        println!(r"\end{{tabular}}")
    }
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        
        self.input.clear();
        self.stack = vec![0];
    }

    pub fn input(self, input: Vec<TV>) -> Self {
        Self {
            input,
            
            action_table: self.action_table,
            goto_table: self.goto_table,
            stack: self.stack,
            rule_table: self.rule_table,
            value_stack: Vec::new(),
            reduce_action_table: self.reduce_action_table,
        }
    }

    pub fn install_reduce_action(self,functions:BTreeMap<LR0Item<NT,T>,Box<dyn Fn(&[ValueStackSymbol<NTV,TV>])->NTV>>)->Self{
       println!("installed for {:?}",functions.keys());
        Self{ input: self.input, action_table: self.action_table, goto_table: self.goto_table, stack: self.stack, rule_table: self.rule_table, value_stack: self.value_stack, reduce_action_table: functions }
    }

    pub fn export_parsing_as_latex_src(&mut self) {
        println!("generating step by step parsing for {:?}.\n", self.input.iter().map(|x|{x.into_kind()}).collect::<Vec<_>>());

        println!("\\begin{{tabular}}{{lllll}}");
        println!(r" & &remain input & stack & action \\ \hline");

        let mut step_count = 1;

        loop {
            if let Some(x) = self.input.get(0) {
                let top_index = self.stack.len();
                let q = *self.stack.get(top_index - 1).unwrap();
                if let Some(action) = self.action_table.get(&(q, x.into_kind())) {
                    match action {
                        ActionKind::Accept => {
                            println!(
                                "{} & & {} & {} & Accept \\\\ \\hline ",
                                step_count,
                                self.dump_remain_input(),
                                self.dump_stack_as_latex_src(),
                            );
                            break;
                        }
                        ActionKind::Reduce(rule_number) => {
                            println!(
                                "{} & & {} & {} & Reduce($ r_{{{}}} $) \\\\ \\hline",
                                step_count,
                                self.dump_remain_input(),
                                self.dump_stack_as_latex_src(),
                                rule_number
                            );

                            if let Some(lr0item) = self.rule_table.get(*rule_number) {
                                let pops = lr0item.right.len();
                                for _ in 0..pops {
                                    self.stack.pop();
                                }
                                let top_index = self.stack.len() - 1;
                                if let Some(q) = self.stack.get(top_index) {
                                    let a = lr0item.left.clone();
                                    let goto_key = (*q, a);
                                    if let Some(q_dash) = self.goto_table.get(&goto_key) {
                                        self.stack.push(*q_dash);

                                        if let Some(function) = self.reduce_action_table.get(lr0item){
                                            let ln =self.value_stack.len();
                                            let args=self.value_stack.split_off(ln-lr0item.dot_pos);
                                            let v=function(&args);
                                            self.value_stack.push(ValueStackSymbol::NonTerm(v));
                                        }else{
                                            panic!("undefined reduce action. for {:?}",lr0item);
                                        }     
                                    } else {
                                        panic!("({},{:?}) -> ?", q, lr0item.left);
                                    }
                                } else {
                                    panic!("stack is empty this is not acceptable.");
                                }
                            } else {
                                panic!("can't get r{} from rule_table", rule_number);
                            }
                        }

                        ActionKind::Shift(next_state) => {
                            println!(
                                "{} & & {} & {} & Shift($ q_{{{}}} $) \\\\ \\hline",
                                step_count,
                                self.dump_remain_input(),
                                self.dump_stack_as_latex_src(),
                                next_state
                            );
                            self.stack.push(*next_state);
                            let value=self.input.remove(0);
                            self.value_stack.push(ValueStackSymbol::Term(value));
                        }
                        ActionKind::Error => {
                            eprintln!("error detected due to invalid input");
                        }
                    }
                    step_count += 1;
                } else {
                    eprintln!("No action for ({},{:?})", q, x.into_kind());
                }
            }
        }

        println!("\\end{{tabular}}")
    }

    fn dump_remain_input(&self) -> String {
        use std::fmt::Write;
        let mut buffer = String::new();
        for x in self.input.iter().map(|tv|{tv.into_kind()}) {
            write!(&mut buffer, "{:?}", x).unwrap();
        }
        buffer
    }

    fn dump_stack_as_latex_src(&self) -> String {
        use std::fmt::Write;

        let mut buffer = String::new();
        buffer.push_str("$ ");
        let (last, left) = self.stack.split_last().unwrap();
        for state in left {
            write!(&mut buffer, "q_{{{}}}", state).unwrap();
        }
        write!(&mut buffer, "q_{{{}}}\\leftarrow", last).unwrap();
        buffer.push_str(" $");
        buffer
    }

    pub fn get_syntax_tree(&mut self)->Option<ValueStackSymbol<NTV,TV>>{
        self.value_stack.pop()
    } 
}
