use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    node::RcNodeRef,
    tree::{AddNodeError, Tree},
};

/*
 * TreeStore is a in-memory store for the tree.
 * In a real application, this would store in the tree in a database.
 * All tree and node related code would live in a separate crate.
 * We would only expose the TreeStore to the rest of the application.
 *
 */
pub struct TreeStore {
    lock: Arc<RwLock<Tree>>,
}

impl Default for TreeStore {
    fn default() -> Self {
        TreeStore {
            lock: Arc::new(RwLock::new(Tree::default())),
        }
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Tree>>> for AddNodeError {
    fn from(_: PoisonError<RwLockWriteGuard<'_, Tree>>) -> Self {
        AddNodeError {
            message: "failed to get lock".to_string(),
        }
    }
}
impl TreeStore {
    pub fn get_tree(&self) -> Result<Vec<RcNodeRef>, PoisonError<RwLockReadGuard<'_, Tree>>> {
        let tree = self.lock.read()?;
        Ok(Vec::from(&*tree))
    }

    pub fn add_node(
        &self,
        label: String,
        parent_id: Option<i32>,
    ) -> Result<RcNodeRef, AddNodeError> {
        let mut tree = self.lock.write()?;
        (*tree).add_node(label, parent_id)
    }

    // Using this for tests so will allow for dead code
    #[allow(dead_code)]
    pub fn len(&self) -> i32 {
        let tree = self.lock.read().unwrap();
        (*tree).len() as i32
    }
}

// TODO: add tests that check if the store is thread safe
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn creates_default_tree() {
        let tree_provider = TreeStore::default();
        let tree = tree_provider.get_tree().unwrap();
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn gets_tree() {
        let tree_provider = TreeStore::default();
        let tree = tree_provider.get_tree().unwrap();
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn adds_node() {
        let tree_provider = TreeStore::default();
        let result = tree_provider.add_node("test".to_string(), None);
        assert!(result.is_ok());
        let tree = tree_provider.get_tree().unwrap();
        assert_eq!(tree.len(), 1);
    }
    //
    // #[test]
    // fn handles_multi_thread_access() {
    //     let tree_provider = TreeStore::default();
    //     thread::scope(|s| {
    //         s.spawn(|| {
    //             println!("first thread");
    //             tree_provider.add_node("root".to_string(), None).unwrap();
    //         });
    //         s.spawn(|| {
    //             tree_provider.add_node("root".to_string(), None).unwrap();
    //         });
    //     });
    //     let tree = tree_provider.get_tree().unwrap();
    //     assert_eq!(tree.len(), 2)
    // }
}
