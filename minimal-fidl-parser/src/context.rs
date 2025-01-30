#[allow(unused_imports)] // So that I don't need to keep adding or removing whilst testing
use super::Key;
use crate::{
    cache::{DirectLeftRecursionCache, Head, IndirectLeftRecursionCache, LR},
    publisher::{DirectLeftRecursionPublisher, IndirectLeftRecursionPublisher},
    Rules,
};
use std::collections::BTreeSet;

use super::{BasicCache, BasicPublisher};

pub trait Context
where
    Self::C: core::fmt::Debug,
    Self::P: core::fmt::Debug,
{
    // Associated types so I can tie specific Cache/Publisher pairs together since they can be mutually exclusive
    // I.e NOOP Cache prohibits IndirectLeftRecursionPublisher since you need a cache to do indirect left recursion.
    type C;

    type P;
    #[allow(dead_code)]

    fn new(size_of_source: usize, number_of_rules: usize) -> Self;
    #[allow(dead_code)]
    fn print_cache(&self);
    #[allow(dead_code)]

    fn print_publisher(&self);
    fn check(&self, rule: Rules, start_position: u32) -> Option<(bool, u32, Key)>;
    fn check_lr(&self, rule: Rules, start_position: u32) -> Option<(bool, u32, Key, LR)>;

    fn connect(&mut self, parent_key: Key, child_key: Key);
    fn connect_front(&mut self, parent_key: Key, child_key: Key);
    fn reserve_publisher_entry(&mut self, rule: Rules) -> Key;
    fn create_cache_entry(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
    );
    fn connect_if_not_connected(&mut self, parent_index: Key, child_index: Key);
    fn create_cache_entry_direct_lr(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
        lr: LR,
    );

    fn update_publisher_entry(
        &mut self,
        key: Key,
        is_true: bool,
        start_position: u32,
        end_position: u32,
    );

    fn check_head(&self, rule: Rules, start_position: u32) -> Option<&Head>;
    fn set_head(&mut self, start_position: u32, head_rule: Rules, involved_set: BTreeSet<Rules>);
    fn rule_in_eval_set(&self, head_index: (Rules, u32), rule: Rules) -> bool;
    fn rule_in_involved_set(&self, head: (Rules, u32), rule: Rules) -> bool;

    fn remove_from_eval_set(&mut self, head_index: (Rules, u32), rule: Rules);
    fn reinitialize_eval_set(&mut self, rule: Rules, start_position: u32);
    fn get_publisher(self) -> Self::P;
    fn clear_node_of_children(&mut self, node: Key);
    fn eval_set_is_empty(&self, start_position: u32, rule: Rules) -> bool;
    fn reset_head(&mut self, rule: Rules, start_position: u32);
    fn disconnect(&mut self, parent: Key, child: Key);
    fn print_node(&self, node: Key);
    fn get_current_active_lr_position(&self) -> Option<(Rules, u32)>;
    fn set_current_active_lr_position(&mut self, position: Option<(Rules, u32)>);
}

pub struct BasicContext {
    cache: BasicCache,
    publisher: BasicPublisher,
}
#[allow(dead_code)]

pub struct DirectLeftRecursionContext {
    cache: DirectLeftRecursionCache,
    publisher: DirectLeftRecursionPublisher,
}
#[allow(dead_code)]

pub struct IndirectLeftRecursionContext {
    cache: IndirectLeftRecursionCache,
    publisher: IndirectLeftRecursionPublisher,
}

impl Context for BasicContext {
    type C = BasicCache;
    type P = BasicPublisher;

    fn new(size_of_source: usize, number_of_rules: usize) -> Self {
        BasicContext {
            cache: Self::C::new(),
            publisher: Self::P::new(size_of_source, number_of_rules),
        }
    }
    fn get_current_active_lr_position(&self) -> Option<(Rules, u32)> {
        self.cache.get_current_active_lr_position()
    }
    fn set_current_active_lr_position(&mut self, position: Option<(Rules, u32)>) {
        self.cache.set_current_active_lr_position(position);
    }

    fn reinitialize_eval_set(&mut self, rule: Rules, start_position: u32) {
        self.cache.reinitialize_eval_set(rule, start_position);
    }
    #[allow(dead_code)]
    fn print_cache(&self) {
        println!("{:?}", &self.cache)
    }
    fn eval_set_is_empty(&self, start_position: u32, rule: Rules) -> bool {
        self.cache.eval_set_is_empty(rule, start_position)
    }
    fn clear_node_of_children(&mut self, node: Key) {
        self.publisher.clear_node_of_children(node);
    }
    fn disconnect(&mut self, parent: Key, child: Key) {
        self.publisher.disconnect(parent, child);
    }
    // fn get_currently_active_head(&self) -> Option<u32> {
    //     self.cache.get_currently_active_head()
    // }
    // fn set_currently_active_head(&mut self, position: Option<u32>) {
    //     self.cache.set_currently_active_head(position);
    // }
    fn print_publisher(&self) {
        //self.publisher.print(Key(0), Some(true));
        println!("\n\n{:?}", &self.publisher)
    }
    fn print_node(&self, node: Key) {
        self.publisher.print(node, None);
    }

    fn reserve_publisher_entry(&mut self, rule: Rules) -> Key {
        self.publisher.add_node(rule, 0, 0, false)
    }
    fn connect(&mut self, parent_key: Key, child_key: Key) {
        // println!(
        //     "Connecting Child: {:?} to Parent: {:?}",
        //     child_key, parent_key
        // );
        self.publisher.connect(parent_key, child_key);
    }
    fn connect_front(&mut self, parent_key: Key, child_key: Key) {
        self.publisher.connect_front(parent_key, child_key);
    }
    fn create_cache_entry(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
    ) {
        self.cache
            .insert(rule, is_true, start_position, end_position, key);
    }
    fn create_cache_entry_direct_lr(
        &mut self,
        rule: Rules,
        is_true: bool,
        start_position: u32,
        end_position: u32,
        key: Key,
        lr: LR,
    ) {
        self.cache
            .insert_direct_lr(rule, is_true, start_position, end_position, key, lr);
    }

    fn update_publisher_entry(
        &mut self,
        key: Key,
        is_true: bool,
        start_position: u32,
        end_position: u32,
    ) {
        self.publisher.set_node_result(key, is_true);
        self.publisher.set_node_start_position(key, start_position);
        self.publisher.set_node_end_position(key, end_position);
    }
    fn check(&self, rule: Rules, start_position: u32) -> Option<(bool, u32, Key)> {
        self.cache.check(rule, start_position)
    }
    fn check_lr(&self, rule: Rules, start_position: u32) -> Option<(bool, u32, Key, LR)> {
        self.cache.check_direct_lr(rule, start_position)
    }
    fn check_head(&self, rule: Rules, start_position: u32) -> Option<&Head> {
        self.cache.check_head(rule, start_position)
    }
    // fn remove_head(&mut self, start_position: u32) {
    //     println!("REMOVE HEAD! {:?}\x1b[0m", (start_position));

    //     self.cache.remove_head(start_position);
    // }
    fn rule_in_involved_set(&self, head: (Rules, u32), rule: Rules) -> bool {
        self.cache.rule_in_involved_set(head, rule)
    }
    fn connect_if_not_connected(&mut self, parent_index: Key, child_index: Key) {
        self.publisher
            .connect_if_not_connected(parent_index, child_index);
    }

    fn reset_head(&mut self, rule: Rules, start_position: u32) {
        println!("RESETTING HEAD!");
        self.cache.reset_head(rule, start_position);
    }
    fn set_head(&mut self, start_position: u32, head_rule: Rules, involved_set: BTreeSet<Rules>) {
        println!("SETTING HEAD");
        self.cache.set_head(start_position, head_rule, involved_set);
    }
    fn rule_in_eval_set(&self, head_index: (Rules, u32), rule: Rules) -> bool {
        self.cache.rule_in_eval_set(head_index, rule)
    }
    fn remove_from_eval_set(&mut self, head_index: (Rules, u32), rule: Rules) {
        self.cache.remove_from_eval_set(head_index, rule);
    }

    fn get_publisher(self) -> Self::P {
        self.publisher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    #[test]
    fn test() {
        let context = BasicContext {
            cache: BasicCache::new(),
            publisher: BasicPublisher::new(0, 0),
        };
        context.print_cache();
        context.print_publisher();
    }
    #[test]
    fn test3() {
        let context = BasicContext::new(0, 0);
        context.print_cache();
        context.print_publisher();
    }

    #[test]
    fn test2() {
        let context: RefCell<BasicContext> = RefCell::new(BasicContext::new(0, 0));
        let borrowed_context = context.borrow();
        borrowed_context.print_cache();
        borrowed_context.print_publisher();
    }
}
