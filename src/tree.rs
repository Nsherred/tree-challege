use crate::node::{as_rc_ref, Node, RcNodeRef};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Tree {
    next_id: i32,
    // For now this will double as a in-memory store, where the node id is 1 + the node's index.
    lookup: HashMap<i32, RcNodeRef>,
    // enforcing that child can only have one parent, to prevent the tree from becoming a graph.
    // If this was a database we would have a unique constraint on the child_id column.
    // Its faster to track at insertion time than to check on every query.
    child_to_parent: HashMap<i32, i32>,
    parent_to_child: HashMap<i32, Vec<i32>>,
}

impl Default for Tree {
    fn default() -> Self {
        Tree {
            child_to_parent: HashMap::new(),
            parent_to_child: HashMap::new(),
            lookup: HashMap::new(),
            next_id: 1,
        }
    }
}

#[derive(Debug)]
pub struct AddNodeError {
    pub message: String,
}

impl AddNodeError {
    fn new(message: String) -> Self {
        AddNodeError { message }
    }
}

impl Tree {
    pub fn add_node(
        &mut self,
        label: String,
        parent_id: Option<i32>,
    ) -> Result<RcNodeRef, AddNodeError> {
        let id = self.next_id;
        let node = as_rc_ref(Node::new(id, label));
        if let Some(parent_id) = parent_id {
            self.add_edge(parent_id, node.clone())?;
        }
        self.lookup.insert(id, node.clone());
        self.next_id = id + 1;
        return Ok(node.clone());
    }

    fn add_edge(&mut self, parent_id: i32, child_ref: RcNodeRef) -> Result<(), AddNodeError> {
        let child = child_ref.lock().unwrap();
        if parent_id == child.id {
            return Err(AddNodeError::new(format!(
                "Cannot add connection, parent and child are the same node: {}",
                parent_id
            )));
        }

        if self.child_to_parent.contains_key(&child.id) {
            return Err(AddNodeError::new(format!(
                "Cannot add connection, child {} already has a parent",
                child.id
            )));
        }

        // we could turn this into a map lookup by changing the way we store nodes from a vec to a
        // hashmap
        if let None = self.lookup.get(&parent_id) {
            return Err(AddNodeError::new(format!(
                "Cannot add connection, parent {} does not exist",
                parent_id
            )));
        };

        self.child_to_parent.insert(child.id, parent_id);
        let mut parent = self.lookup[&parent_id].lock().unwrap();
        parent.add_child(child_ref.clone());
        self.parent_to_child
            .entry(parent_id)
            .or_insert(vec![])
            .push(child.id);
        return Ok(());
    }

    pub fn len(&self) -> i32 {
        self.lookup.keys().len() as i32
    }

    pub fn get_node(&self, index: &i32) -> Option<RcNodeRef> {
        match self.lookup.get(index) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

impl<'a> From<&Tree> for Vec<Arc<Mutex<Node>>> {
    fn from(value: &Tree) -> Self {
        let root_ids: Vec<&i32> = value
            .lookup
            .keys()
            .filter(|key| !value.child_to_parent.contains_key(key))
            .collect();
        root_ids
            .into_iter()
            .map(|id| value.lookup[id].clone())
            .collect()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn builds_default_tree() {
        let tree = Tree::default();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.child_to_parent.len(), 0);
        assert_eq!(tree.parent_to_child.len(), 0);
    }

    #[test]
    fn can_add_node_to_tree() {
        let mut tree = Tree::default();
        tree.add_node("root".to_string(), None).unwrap();
        assert_eq!(tree.len(), 1);
        let node = tree.lookup[&1].lock().unwrap();
        assert_eq!(node.id, 1);
        assert_eq!(node.label, "root");
    }

    #[test]
    fn can_add_node_to_tree_with_parent() {
        let mut tree = Tree::default();
        tree.add_node("root".to_string(), None).unwrap();
        tree.add_node("child".to_string(), Some(1)).unwrap();
        assert_eq!(tree.len(), 2);
        let arc = tree.get_node(&2).unwrap();
        let node = arc.lock().unwrap();
        assert_eq!(node.id, 2);
        assert_eq!(node.label, "child");
        assert_eq!(tree.child_to_parent.len(), 1);
        assert_eq!(tree.parent_to_child.len(), 1);
        assert_eq!(tree.parent_to_child.get(&1).unwrap().len(), 1);
    }

    #[test]
    fn edge_errors_propagate() {
        let mut tree = Tree::default();
        let result = tree.add_node("root".to_string(), Some(2));
        assert!(result.is_err());
    }

    #[test]
    fn can_add_connections_to_tree() {
        let mut tree = Tree::default();
        tree.add_node("root".to_string(), None).unwrap();
        let child = tree.add_node("child".to_string(), None).unwrap();
        tree.add_edge(1, child).unwrap();
        assert_eq!(tree.child_to_parent.len(), 1);
        assert_eq!(tree.parent_to_child.len(), 1);
        assert_eq!(tree.parent_to_child.get(&1).unwrap().len(), 1);
    }

    #[test]
    fn cannot_add_edge_with_self() {
        let mut tree = Tree::default();
        let parent = tree.add_node("root".to_string(), None).unwrap();

        let result = tree.add_edge(1, parent);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_override_edge() {
        let mut tree = Tree::default();
        tree.add_node("root".to_string(), None).unwrap();
        let child = tree.add_node("child".to_string(), Some(1)).unwrap();
        tree.add_node("child".to_string(), Some(1)).unwrap();
        let result = tree.add_edge(3, child);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_add_edge_with_nonexistent_parent() {
        let mut tree = Tree::default();
        let node = tree.add_node("root".to_string(), None).unwrap();
        let result = tree.add_edge(3, node);
        assert!(result.is_err());
    }

    #[test]
    fn transforms_into() {
        let mut tree = Tree::default();

        tree.add_node("root".to_string(), None).unwrap();
        tree.add_node("child".to_string(), Some(1)).unwrap();
        let nodes = Vec::<RcNodeRef>::from(&tree);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].lock().unwrap().len(), 1);
    }

    #[test]
    fn transforms_into_with_multiple_children() {
        let mut tree = Tree::default();

        tree.add_node("root".to_string(), None).unwrap();
        tree.add_node("child".to_string(), Some(1)).unwrap();
        tree.add_node("child".to_string(), Some(1)).unwrap();
        let nodes = Vec::<RcNodeRef>::from(&tree);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].lock().unwrap().len(), 2);
    }
}
