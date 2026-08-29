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
use log::info;

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
        let prefix = self.find_longest_prefix_path(token);

        info!("Prefix is: {:?}", prefix);

        let (bytes, mut node_index) = match prefix {
            Some(prefix) => (&token.as_bytes()[prefix.len()..], **prefix.last().expect("")),
            None => (token.as_bytes(), self.root_id) 
        };

        for byte in bytes {
            let next_node = Node::new(&byte);
            self.children
                .entry(node_index)
                .or_insert_with(Vec::new)
                .push(next_node.id);
            node_index = next_node.id;
            self.nodes.insert(next_node.id, next_node);
            info!("Inserting node for {:?}", byte);
        }
    }

    pub fn find_longest_prefix_path(&mut self, token: &str) -> Option<Vec<&u64>> {
        let mut bytes = token.bytes();
        let first_byte = bytes.next().unwrap();
        let first_node = match self.children.get(&self.root_id) {
            Some(children) => {
                let root_children = children.iter().map(|id| self.nodes.get(id)).collect::<Vec<Option<&Node>>>();
                info!("Root children are: {:?}", root_children);
                info!("Matches: {:?}", root_children[0].unwrap().value == first_byte);
                children.iter().find(|id| self.nodes.get(id).expect("All nodes must have an ID; something is wrong").value == first_byte)
            },
            None => return None,
        };

        info!("Made it this far");

        let mut current_node = first_node.expect("current node has to exist");
        let mut path = vec![current_node];
        for byte in bytes {
            let current_children = self.children.get(current_node);
            match current_children {
                None => return Some(path),
                Some(children) => {
                    info!("Made it to current children");
                    let next_node = children
                        .iter()
                        .find(|id| self.nodes.get(id).expect("All nodes must have an ID; something is wrong").value == byte);
                   
                    if let Some(value) = next_node {
                        path.push(value);
                        current_node = value;
                        info!("checkpoint 1");
                        continue;
                    }

                    info!("There are children, but none have the correct value");
                    return None;
                }
            }
        }
        info!("Should return a list here");
        Some(path)
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
        trie.add_token("car");
        info!("Nodes are: {:?}", trie.nodes);
        trie.add_token("cart");
        info!("Children are: {:?}", trie.children);
        assert_eq!(true, true);
    }
}
