use std::fmt::Debug;
use std::option::Option;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TreeNode<T>
where
    T: Clone + Default + Debug,
{
    item: T,
    children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T>
where
    T: Clone + Default + Debug,
{
    fn init_tree(items: Vec<T>) -> Vec<TreeNode<T>> {
        let mut tree_items: Vec<TreeNode<T>> = Vec::new();
        for item in items {
            let tree = TreeNode {
                item,
                ..Default::default()
            };
            tree_items.push(tree);
        }
        tree_items
    }

    fn set_tree<F1, F2>(
        tree_items: Vec<TreeNode<T>>,
        parent_key: String,
        get_key: F1,
        get_parent_key: F2,
    ) -> Vec<TreeNode<T>>
    where
        F1: Fn(&T) -> String + Clone,
        F2: Fn(&T) -> String + Clone,
    {
        let mut trees: Vec<TreeNode<T>> = Vec::new();
        for mut tree in tree_items.clone() {
            if get_parent_key(&tree.item) == parent_key {
                tree.children = Some(Self::set_tree(
                    tree_items.clone(),
                    get_key(&tree.item),
                    get_key.clone(),
                    get_parent_key.clone(),
                ));
                trees.push(tree.clone())
            }
        }
        trees
    }

    pub fn build_tree<F1, F2>(
        items: Vec<T>,
        parent_key: String,
        get_key: F1,
        get_parent_key: F2,
    ) -> Vec<TreeNode<T>>
    where
        F1: Fn(&T) -> String + Clone,
        F2: Fn(&T) -> String + Clone,
    {
        let items = Self::init_tree(items);
        Self::set_tree(items, parent_key, get_key, get_parent_key)
    }
}
