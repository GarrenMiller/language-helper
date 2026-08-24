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
static NEXT_EDGE_ID: AtomicU64 = AtomicU64::new(0);

struct Node {
    id: u64,
    parent_id: Option<u64>,
    outgoing_edge_ids: Vec<u64>,
    is_terminal: bool,
}

#[derive(Debug)]
struct Edge {
    id: u64,
    source_id: u64,
    target_id: u64,
    value: u8,
}

struct Trie {
    root_id: u64,
    nodes: HashMap<u64, Node>,
    edges: HashMap<u64, Edge>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
            parent_id: None,
            outgoing_edge_ids: Vec::new(),
            is_terminal: false,
        }
    }
}

impl Edge {
    pub fn new(value: &u8, source: &u64, target: &u64) -> Self {
        Self {
            id: NEXT_EDGE_ID.fetch_add(1, Ordering::Relaxed),
            source_id: *source,
            target_id: *target,
            value: *value,
        }
    }
}

impl Trie {
    pub fn new() -> Self {
        let root = Node::new();

        let mut instance = Self {
            root_id: root.id,
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };

        instance.nodes.insert(root.id, root);
        return instance;
    }

    pub fn add_token(&mut self, token: &str) {
        let mut bytes = token.bytes();
        let first_byte = bytes.next();
        let mut prev_node = Node::new();
        prev_node.parent_id = Some(self.root_id);

        let edge = Edge::new(&first_byte.unwrap(), &self.root_id, &prev_node.id); 
        let mut parent_id = prev_node.id;
        self.nodes.insert(prev_node.id, prev_node);
        self.edges.insert(edge.id, edge);
        self.nodes
            .get_mut(&self.root_id)
            .unwrap()
            .outgoing_edge_ids
            .push(self.root_id);


        for byte in bytes {
            let mut node = Node::new();
            node.parent_id = Some(parent_id);

            let edge = Edge::new(&byte, &parent_id, &node.id);
            self.edges.insert(edge.id, edge);

            parent_id = node.id;
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
        info!("Edges are: {:?}", trie.edges);
        assert_eq!(true, true);
    }
}
