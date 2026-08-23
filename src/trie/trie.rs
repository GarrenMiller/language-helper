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

struct Edge {
    value: u8,
    edge_id: Uuid,
    source_id: Uuid,
    target_id: Uuid,
}

struct Trie {
    root: Node,
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
        let root_node = Node::new();
        let mut nodes = HashMap::new();
        let mut instance = Self {
            root: root_node,
            nodes: nodes,
            edges: HashMap::new(), 
        };
        instance.nodes.insert(instance.root.id, instance.root);
        return instance;
    }

    pub fn add_token(&mut self, token: &str) {
        let bytes = token.bytes();
        let first_byte = bytes.next();
        let mut prev_node = Node::new();
        prev_node.parent_id = self.root;
        self.nodes.insert(prev_node.id, prev_node);

        let edge = Edge {
            value: first_byte.unwrap(),
            edge_id: Uuid::new_v4(),
            source_id: self.root.id,
            target_id: prev_node.id
        };

        self.edges.insert(edge.edge_id, edge);

        for byte in token.bytes() {
            let mut node = Node::new();
            node.parent_id = Some(prev_node.id);

            let edge = Edge {
                value: byte,
                edge_id: Uuid::new_v4(),
                source_id: prev_node.id,
                target_id: node.id
            };

            prev_node = node;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn test_trie_add_token() {
        let mut trie = Trie::new();
        trie.add_token("cat");
        assert_eq!(true, true);
    }
}
