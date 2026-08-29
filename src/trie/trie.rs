// Need to build a byte prefix trie. This means a collection of Nodes, which are the postions of
// bytes. There are Edges between Nodes, and each Edge has an associated byte; hence the name.
//
// Nodes can be terminal, meaning they are the end of a path, the traversal of which results in a
// token. Terminal nodes have no children.
//
// Nodes that are not terminal and have no children are dead-end Nodes.
//
// There is the root Node, which has no parent and is not terminal. 
//
// Nodes only have one incoming edge, can have many outgoing
//
// Necessary functions:
// 1. Get valid next bytes for a given prefix.
// 2. Get all tokens that contain a prefix.
// 3. Add new token in and update paths accordingly.
//
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct Node {
    id: u64,
    value: u8,
    is_terminal: bool,
}

pub struct Trie {
    root_id: u64,
    nodes: HashMap<u64, Node>,
    children: HashMap<u64, Vec<u64>>
}

impl Node {
    pub fn new(byte: &u8) -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
            value: *byte,
            is_terminal: false,
        }
    }
}


impl Trie {
    pub fn new() -> Self {
        let root = Node::new(&b'"');

        let mut instance = Self {
            root_id: root.id,
            nodes: HashMap::new(),
            children: HashMap::new(),
        };

        instance.nodes.insert(root.id, root);
        return instance;
    }

    pub fn add_token(&mut self, token: &str) {
        let mut bytes = token.bytes();
        let first_byte = bytes.next().unwrap();
        let first_node = Node::new(&first_byte);

        let mut parent_id = first_node.id;
        self.children
            .entry(self.root_id)
            .or_insert_with(Vec::new)
            .push(first_node.id);
        self.nodes.insert(first_node.id, first_node);

        for byte in bytes {
            let next_node = Node::new(&byte);
            parent_id = next_node.id;
            self.children
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(next_node.id);
            self.nodes.insert(next_node.id, next_node);
        }
    }

    pub fn find_longest_prefix_path(&mut self, token: &str) {
        let mut bytes = token.bytes();
        let first_byte = bytes.next().unwrap();
        let first_node = self.children
            .get(&self.root_id)
            .expect("Do not try to access root node children before they exist")
            .iter()
            .find(|id| self.nodes.get(id).expect("All nodes must have an ID; something is wrong").value == first_byte);

        if first_node.is_none() {
            println!("TODO: Return empty string to indicate that no prefix is known");
            return;
        }

        let mut current_node = first_node.expect("current node has to exist");
        let mut path = vec![current_node];
        for byte in bytes {
            let current_children = self.children.get(current_node);
            match current_children {
                None => println!("There are no children"),
                Some(children) => {
                    let next_node = children
                        .iter()
                        .find(|id| self.nodes.get(id).expect("All nodes must have an ID; something is wrong").value == byte);
                   
                    if let Some(value) = next_node {
                        path.push(value);
                        current_node = value;
                        return;
                    }

                    println!("There are children, but none have the correct value");
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use log::info;

 
    #[test]
    fn test_trie_add_token() {
        let mut trie = Trie::new();
        trie.add_token("cat");
        info!("Nodes are: {:?}", trie.nodes);
        info!("Children are: {:?}", trie.children);
        assert_eq!(true, true);
    }
}
