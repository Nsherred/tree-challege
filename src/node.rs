use std::sync::{Arc, Mutex};

use serde::Serialize;

pub type RcNodeRef = Arc<Mutex<Node>>;

#[derive(Serialize)]
pub struct Node {
    pub id: i32,
    pub label: String,
    children: Vec<RcNodeRef>,
}

pub fn as_rc_ref(node: Node) -> RcNodeRef {
    Arc::new(Mutex::new(node))
}

impl Node {
    pub fn new(id: i32, label: String) -> Self {
        Node {
            id,
            label,
            children: vec![],
        }
    }
    pub fn new_with_children(id: i32, label: String, children: Vec<RcNodeRef>) -> Self {
        Node {
            id,
            label,
            children,
        }
    }

    pub fn add_child(&mut self, child: RcNodeRef) {
        self.children.push(child);
    }

    pub fn len(&self) -> i32 {
        self.children.len() as i32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builds_recursive_node_tree() {
        let mut parent = Node::new(1, "root".to_string());
        let child = as_rc_ref(Node::new(2, "child".to_string()));
        parent.add_child(child);
        let actual_child = parent.children[0].lock().unwrap();
        assert_eq!(actual_child.id, 2);
        assert_eq!(actual_child.label, "child");
    }

    #[test]
    fn serializes_recursive_node_tree() {
        let mut node = Node {
            id: 1,
            label: "root".to_string(),
            children: vec![],
        };
        let node2 = as_rc_ref(Node {
            id: 2,
            label: "child".to_string(),
            children: vec![],
        });
        node.add_child(node2);
        let json = serde_json::to_string(&node).unwrap();
        assert_eq!(
            json,
            r#"{"id":1,"label":"root","children":[{"id":2,"label":"child","children":[]}]}"#
        );
    }
}
