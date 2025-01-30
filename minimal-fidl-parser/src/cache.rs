use crate::{Key, Rules};
use std::collections::VecDeque;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone)]
pub struct Head {
    pub start_position: u32,
    pub active_left_recursion_rule: Rules,
    pub involved_set: BTreeSet<Rules>,
    pub eval_set: BTreeSet<Rules>,
}
#[derive(Debug, Copy, Clone)]
pub enum LR {
    Set,
    Unset,
}

#[derive(Debug)]
pub struct BasicCache {
    cache: HashMap<(Rules, u32), (bool, u32, Key)>,
    left_recursion_cache: HashMap<(Rules, u32), (bool, u32, Key, LR)>,
    heads: HashMap<(Rules, u32), Head>,
    current_active_left_recursion: Option<(Rules, u32)>,
}

impl BasicCache {
    pub fn new() -> Self {
        BasicCache {
            cache: HashMap::new(),
            left_recursion_cache: HashMap::new(),
            heads: HashMap::new(),
            current_active_left_recursion: None,
        }
    }

    pub fn get_current_active_lr_position(&self) -> Option<(Rules, u32)> {
        println!(
            "Getting Current Active LR Position: {:?}",
            self.current_active_left_recursion
        );
        self.current_active_left_recursion
    }
    pub fn set_current_active_lr_position(&mut self, active_lr: Option<(Rules, u32)>) {
        println!("Setting Current Active LR Position: {:?}", active_lr);
        self.current_active_left_recursion = active_lr;
    }

    pub fn insert(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
    ) {
        let x = self
            .cache
            .insert((rule, start_position), (is_true, end_position, key));
        match x {
            Some(x) => {
                //println!("Overwriting Key: {:?} with Key: {:?}", x.2, key)
            }
            None => {}
        }
    }
    pub fn check(&self, rule: Rules, start_position: u32) -> Option<(bool, u32, Key)> {
        self.cache.get(&(rule, start_position)).copied()
    }
    pub fn insert_direct_lr(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
        lr: LR,
    ) {
        self.left_recursion_cache
            .insert((rule, start_position), (is_true, end_position, key, lr));
    }
    pub fn check_direct_lr(
        &self,
        rule: Rules,
        start_position: u32,
    ) -> Option<(bool, u32, Key, LR)> {
        self.left_recursion_cache
            .get(&(rule, start_position))
            .copied()
    }

    pub fn check_head(&self, rule: Rules, start_position: u32) -> Option<&Head> {
        let head = self.heads.get(&(rule, start_position));
        #[allow(clippy::manual_map)]
        match head {
            None => None, // Not yet exists therefore no Left Recursion at this position
            Some(head) => Some(&head), // Return head rule since that's necessary information.
        }
    }
    // pub fn remove_head(&mut self, start_position: u32) {
    //     // println!("IN REMOVE HEAD");
    //     // println!("Recursion Stack: {:?}", self.recursion_stack);

    //     // Remove head
    //     self.heads.remove(&start_position);
    //     // match self.recursion_stack.pop_front() {
    //     //     None => {}
    //     //     Some(head) => {
    //     //         self.heads.insert(start_position, head);
    //     //     }
    //     // };
    // }
    pub fn reset_head(&mut self, rule: Rules, start_position: u32) {
        // println!("RESET HEAD!");
        // println!("\x1b[0m");
        //self.heads.remove(&start_position); // Remove this head
        // let past_head = self.recursion_stack.pop_front();

        // match past_head {
        //     None => {}
        //     Some(past_head) => {
        //         // println!("REINSERTING HEAD RECURSION");
        //         self.heads.insert((rule, start_position), past_head);
        //     }
        // };
    }

    pub fn set_head(
        &mut self,
        start_position: u32,
        head_rule: Rules,
        involved_set: BTreeSet<Rules>,
    ) {
        // println!("\x1b[31m");
        // println!("SET HEAD! {:?}", (start_position, head_rule));

        // println!("Recursion Stack: {:?}", self.recursion_stack);
        // let current_active_lr_position = self.get_current_active_lr_position();
        // println!(
        //     "Current Active LR Position in Set Head: {:?}",
        //     current_active_lr_position
        // );
        // match current_active_lr_position {
        //     Some(active_lr_position) => {
        //         let old_value = self
        //             .heads
        //             .remove(&active_lr_position)
        //             .expect("If Active LR position valid then head should exist!");
        //         self.recursion_stack.push_front(old_value);
        //     }
        //     None => {}
        // }
        let eval_set = involved_set.clone();
        let head = Head {
            start_position,
            active_left_recursion_rule: head_rule,
            involved_set,
            eval_set,
        };
        self.heads.insert((head_rule, start_position), head);
    }

    pub fn reinitialize_eval_set(&mut self, rule: Rules, start_position: u32) {
        let head = self
            .heads
            .get_mut(&(rule, start_position))
            .expect("Should exist by now");
        head.eval_set = head.involved_set.clone();
        // println!("Reinit Eval Set: {:?}", head.eval_set);
    }

    pub fn eval_set_is_empty(&self, rule: Rules, start_position: u32) -> bool {
        let head = self
            .heads
            .get(&(rule, start_position))
            .expect("Should always exist when calling rule_in_eval_set");
        // println!("Rule: {:?}\nEval Set: {:?}", rule, head.eval_set);
        head.eval_set.is_empty()
    }
    pub fn rule_in_involved_set(&self, head_index: (Rules, u32), rule: Rules) -> bool {
        // Return true if it's in eval set

        let head = self
            .heads
            .get(&head_index)
            .expect("Should always exist when calling rule_in_involved_set");
        // println!("Rule: {:?}\nEval Set: {:?}", rule, head.eval_set);
        let is_involved = head.involved_set.contains(&rule);
        println!(
            "Rule in involved set: Rule : {:?}, Start Position: {:?}, {:?}",
            rule, head_index.1, is_involved
        );
        is_involved
    }

    pub fn rule_in_eval_set(&self, head_index: (Rules, u32), rule: Rules) -> bool {
        // Return true if it's in eval set
        let head = self
            .heads
            .get(&head_index)
            .expect("Should always exist when calling rule_in_eval_set");
        // println!("Rule: {:?}\nEval Set: {:?}", rule, head.eval_set);
        head.eval_set.contains(&rule)
    }
    pub fn remove_from_eval_set(&mut self, head_index: (Rules, u32), rule: Rules) {
        self.heads
            .get_mut(&head_index)
            .expect("Should exist by now")
            .eval_set
            .remove(&rule);
    }
}

#[allow(dead_code)]
#[derive(Debug)]

pub struct DirectLeftRecursionCache;
#[allow(dead_code)]
#[derive(Debug)]

pub struct IndirectLeftRecursionCache;
