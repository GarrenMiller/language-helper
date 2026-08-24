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
use uuid::Uuid;
use std::collections::HashMap;

struct Node {
    id: Uuid,
    parent_id: Option<Uuid>,
    outgoing_edges: Vec<Edge>,
    is_terminal: bool,
}

#[derive(Debug)]
struct Edge {
    value: u8,
    edge_id: Uuid,
    source_id: Uuid,
    target_id: Uuid,
}

struct Trie {
    root_id: Uuid,
    nodes: HashMap<Uuid, Node>,
    edges: HashMap<Uuid, Edge>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            outgoing_edges: Vec::new(),
            is_terminal: false,
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

        let edge = Edge {
            value: first_byte.unwrap(),
            edge_id: Uuid::new_v4(),
            source_id: self.root_id,
            target_id: prev_node.id
        };

        let mut parent_id = prev_node.id;
        self.nodes.insert(prev_node.id, prev_node);
        self.edges.insert(edge.edge_id, edge);

        for byte in token.bytes() {
            let mut node = Node::new();
            node.parent_id = Some(parent_id);

            let edge = Edge {
                value: byte,
                edge_id: Uuid::new_v4(),
                source_id: parent_id,
                target_id: node.id
            };

            self.edges.insert(edge.edge_id, edge);

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
    
    #[test]
    fn test_logging() {
        info!("Logging works");
        assert_eq!(true, true);
    }
}
