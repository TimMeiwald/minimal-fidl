use num::range;

use crate::Key;
use crate::Rules;
// Tree needs to be able to swap existing structures into other locations
// E.g Rule A may fail but calls subrule B where B succeeds
// Then Rule C may try on same start location and also need subrule B
// So rather than redo subrule B we swap in the existing cached subrule B value.

#[derive(Debug)]

pub struct BasicPublisher {
    nodes: Vec<Node>,
}

impl PartialEq for BasicPublisher {
    fn eq(&self, other: &Self) -> bool {
        self.partial_eq_kernel(other, Key(0), Key(0))
    }
}

impl BasicPublisher {
    fn partial_eq_kernel(&self, other: &Self, self_index: Key, other_index: Key) -> bool {
        let self_node = self.get_node(self_index);
        let other_node = other.get_node(other_index);
        if self_node == other_node {
            for index in range(0, self_node.children.len()) {
                if self_node.get_children().len() != other_node.get_children().len() {
                    return false;
                }
                let self_child_key = self_node.get_children()[index];
                let other_child_key = other_node.get_children()[index];
                if !self.partial_eq_kernel(other, self_child_key, other_child_key) {
                    return false;
                }
            }
            return true;
        } else {
            return false;
        }
    }
    pub fn clear_node_of_children(&mut self, node: Key) {
        self.get_mut_node(node).set_children(vec![]);
    }

    pub fn new(size_of_source: usize, number_of_rules: usize) -> Self {
        //let memory_to_allocate = ((size_of_source * number_of_rules) * 64) / 128;
        // println!("Publisher Allocating {} KB", {
        //     memory_to_allocate / (1024)
        // });
        let mut p = BasicPublisher {
            nodes: Vec::<Node>::with_capacity((size_of_source * number_of_rules) / 64),
        };
        p.add_node(Rules::Grammar, 0, 0, true);
        p
    }

    pub fn clear_tree(&mut self) {
        self.nodes.clear();
    }

    pub fn add_node(
        &mut self,
        rule: Rules,
        start_position: u32,
        end_position: u32,
        result: bool,
    ) -> Key {
        let len = self.nodes.len();
        let key = Key(len.try_into().unwrap());
        let node = Node {
            index: key,
            rule,
            start_position,
            end_position,
            result,
            children: Vec::new(),
        };
        self.nodes.push(node);
        key
    }

    pub fn connect(&mut self, parent_index: Key, child_index: Key) {
        //println!("Connecting: {:?} <- {:?}", parent_index, child_index);
        debug_assert!(
            parent_index != child_index,
            "Debug: Cannot connect a Node to itself! -> In BasicPublisher.connect, Parent: {:?}, Child: {:?}", parent_index, child_index
        );

        let parent_node: &mut Node = self.get_mut_node(parent_index);
        parent_node.children.push(child_index);
    }
    pub fn connect_front(&mut self, parent_index: Key, child_index: Key) {
        // Copy is fine because typically not that many keys and this fn should only be necessary for Left Recursive Rules
        // that are comparably infrequent. Could use a deque or whatever instead but right now I'm just using a Vec
        // pending actual performance testing.
        // There may well be a better way to do things but just to get things working.
        //println!("Connecting: {:?} <- {:?}", parent_index, child_index);
        debug_assert!(
            parent_index != child_index,
            "Debug: Cannot connect a Node to itself! -> In BasicPublisher.connect_front"
        );
        let parent_node: &mut Node = self.get_mut_node(parent_index);
        let mut copy_of_children = parent_node.children.clone();
        parent_node.children.clear();
        parent_node.children.push(child_index);
        parent_node.children.append(&mut copy_of_children);
    }
    pub fn connect_if_not_connected(&mut self, parent_index: Key, child_index: Key) {
        //println!("Connecting: {:?} <- {:?}", parent_index, child_index);
        let parent_node: &mut Node = self.get_mut_node(parent_index);
        for child in &parent_node.children {
            if *child == child_index {
                return;
            }
        }
        parent_node.children.push(child_index);
    }
    pub fn disconnect(&mut self, parent_index: Key, child_index: Key) {
        let parent_node: &mut Node = self.get_mut_node(parent_index);
        let mut remove_pos: Option<usize> = None;
        for (position, child) in parent_node.children.iter().enumerate() {
            if *child == child_index {
                remove_pos = Some(position);
            }
        }
        match remove_pos {
            None => {}
            Some(remove_pos) => {
                parent_node.children.remove(remove_pos);
            }
        }
    }

    pub fn set_node_start_position(&mut self, index: Key, start_position: u32) {
        self.nodes[usize::from(index)].start_position = start_position
    }
    pub fn set_node_end_position(&mut self, index: Key, end_position: u32) {
        self.nodes[usize::from(index)].end_position = end_position
    }
    pub fn set_node_result(&mut self, index: Key, result: bool) {
        self.nodes[usize::from(index)].result = result
    }

    pub fn print(&self, index: Key, boolean: Option<bool>) {
        let indent = 0;
        if boolean.is_some() && boolean.unwrap() {
            self.print_kernel_true(index, indent);
        } else if boolean.is_some() && !boolean.unwrap() {
            self.print_kernel_false(index, indent);
        } else {
            self.print_kernel_all(index, indent);
        }
    }

    pub fn clear_false(&self) -> BasicPublisher {
        // Since Tree is a write only structure we need to remvoe False results to be more useful at some point
        // Consume self and return a new Tree only containing the True results.
        let capacity = self.len() / 4; // Heuristic
        let mut new_tree = BasicPublisher::new(capacity, 1);

        let index = 0;

        let node = self.get_node(Key(index));
        //println!("Clear_False Index: {:?}", index);
        if node.result {
            new_tree.add_node(
                node.rule,
                node.start_position,
                node.end_position,
                node.result,
            );
            for i in &node.children {
                self.clear_false_kernel(&mut new_tree, Key(index), *i);
            }
        }
        new_tree
    }
}

impl BasicPublisher {
    pub fn get_node(&self, index: Key) -> &Node {
        &self.nodes[usize::from(index)]
    }

    fn get_mut_node(&mut self, index: Key) -> &mut Node {
        &mut self.nodes[usize::from(index)]
    }

    fn clear_false_kernel(&self, new_tree: &mut BasicPublisher, parent_index: Key, index: Key) {
        //println!("Clear_False_Kernel Index: {:?}", index);

        let node = self.get_node(index);
        if node.result && (node.start_position != node.end_position) {
            let child_index = new_tree.add_node(
                node.rule,
                node.start_position,
                node.end_position,
                node.result,
            );
            new_tree.connect(parent_index, child_index);
            for i in &node.children {
                self.clear_false_kernel(new_tree, child_index, *i);
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    fn print_kernel_true(&self, index: Key, indent: usize) {
        let node = &self.nodes[usize::from(index)];
        if !node.result {
            return;
        }
        node.print(indent);
        let mut counter = 0;
        loop {
            if counter >= node.children.len() {
                break;
            }
            let child_index = node.children[counter];
            if child_index == Key(0) {
                println!("Error found Key(0) should not happen!");
                break;
            }
            self.print_kernel_true(child_index, indent + 1);
            counter += 1;
        }
    }
    fn print_kernel_false(&self, index: Key, indent: usize) {
        let node = &self.nodes[usize::from(index)];
        if node.result {
            return;
        }
        print!("\x1b[31m");
        node.print(indent);
        print!("\x1b[0m");

        let mut counter = 0;
        loop {
            if counter >= node.children.len() {
                break;
            }
            let child_index = node.children[counter];
            if child_index == Key(0) {
                println!("Error found Key(0) should not happen!");
                break;
            }
            self.print_kernel_false(child_index, indent + 1);
            counter += 1;
        }
    }

    fn print_kernel_all(&self, index: Key, indent: usize) {
        //thread::sleep(Duration::new(0, 5*10_u32.pow(7)));

        let node = &self.nodes[usize::from(index)];
        if !node.result {
            print!("\x1b[31m");
        }
        node.print(indent);
        if !node.result {
            print!("\x1b[0m")
        }
        let mut counter = 0;
        loop {
            if counter >= node.children.len() {
                break;
            }
            let child_index = node.children[counter];
            if child_index == Key(0) {
                println!("Error found Key(0) should not happen!");
                break;
            }
            self.print_kernel_all(child_index, indent + 1);
            counter += 1;
        }
    }
}

#[derive(Debug)]
pub struct Node {
    index: Key,
    pub rule: Rules,
    pub start_position: u32,
    pub end_position: u32,
    pub result: bool,
    children: Vec<Key>,
    // To minimize allocations maybe have a second struct that contains all child indices and have Node just contain a start_child_index and end_child_index
    // Because then we can preallocate a load of memory, means a pointer indirection which may or may not impact performance so needs profiling.
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        // Custom Eq because we don't care about the Key value, We care about
        // Matching rule, start_position, end_position, result and the structure of the children.

        if self.rule != other.rule {
            println!("Rule must match!");
            self.print(0);
            other.print(0);
            return false;
        } else if self.start_position != other.start_position {
            println!("Start Position must match!");
            self.print(0);
            other.print(0);
            return false;
        } else if self.end_position != other.end_position {
            println!("End Position must match!");
            self.print(0);
            other.print(0);
            return false;
        } else if self.result != other.result {
            println!("Result(boolean) must be identical!");
            self.print(0);
            other.print(0);
            return false;
        } else if self.children.len() != other.children.len() {
            println!("Number of children must be identical!.");
            self.print(0);
            other.print(0);
            return false;
        } else {
            return true;
        }
    }
}

impl Node {
    pub fn new(rule: Rules, start_position: u32, end_position: u32, result: bool) -> Self {
        Node {
            index: Key(0),
            rule,
            start_position,
            end_position,
            result,
            children: Vec::<Key>::new(),
        }
    }

    pub fn get_string(&self, source: &str) -> String {
        source[(self.start_position as usize)..(self.end_position as usize)].to_string()
    }

    pub fn get_children(&self) -> &Vec<Key> {
        &self.children
    }
    pub fn set_children(&mut self, children: Vec<Key>) {
        self.children = children;
    }

    pub fn print(&self, indent: usize) {
        println!(
            "{}{:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            "    ".repeat(indent),
            self.index,
            self.rule,
            self.start_position,
            self.end_position,
            self.result,
            self.children.len()
        )
    }
}

#[derive(Debug)]

pub struct DirectLeftRecursionPublisher;
#[derive(Debug)]

pub struct IndirectLeftRecursionPublisher;
