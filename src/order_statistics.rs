use std::{cmp::Ordering, fmt::Debug};

use crate::tree::{Metadata, Node, Side, Tree};

#[derive(Debug, Clone, Copy)]
pub struct OrderStatistics {
    order: usize,
}

impl<K: Ord, V> Metadata<K, V> for OrderStatistics {
    fn update(node: Option<&Node<K, V, Self>>) -> Self {
        let mut this = OrderStatistics { order: 1 };

        if let Some(node) = node {
            if let Some(left) = node.left() {
                this.order += left.metadata().order;
            }
            if let Some(right) = node.right() {
                this.order += right.metadata().order;
            }
        }

        this
    }
}

pub trait OsTreeExt<K: Ord, V> {
    fn find_by_rank(&self, rank: usize) -> Option<&Node<K, V, OrderStatistics>>;
    fn remove_by_rank(&mut self, rank: usize) -> Option<Box<Node<K, V, OrderStatistics>>>;
}

pub trait OsNodeExt<K: Ord, V> {
    fn split_by_rank(
        node: Option<Box<Node<K, V, OrderStatistics>>>,
        rank: usize,
    ) -> (
        Option<Box<Node<K, V, OrderStatistics>>>,
        Option<Box<Node<K, V, OrderStatistics>>>,
    );
}

impl<K: Ord, V> OsTreeExt<K, V> for Tree<K, V, OrderStatistics> {
    fn find_by_rank(&self, rank: usize) -> Option<&Node<K, V, OrderStatistics>> {
        fn find_in_node_by_rank<K: Ord, V>(
            node: Option<&Node<K, V, OrderStatistics>>,
            rank: usize,
        ) -> Option<&Node<K, V, OrderStatistics>> {
            if let Some(node) = node {
                if rank >= node.metadata().order {
                    return None;
                }

                let rank_of_left = node.left().map_or(0, |left| left.metadata().order);

                match rank.cmp(&rank_of_left) {
                    Ordering::Less => find_in_node_by_rank(node.left(), rank),
                    Ordering::Equal => Some(node),
                    Ordering::Greater => {
                        find_in_node_by_rank(node.right(), rank - rank_of_left - 1)
                    }
                }
            } else {
                None
            }
        }

        find_in_node_by_rank(self.root(), rank)
    }

    fn remove_by_rank(&mut self, rank: usize) -> Option<Box<Node<K, V, OrderStatistics>>> {
        let root_box = self.root_box_mut();
        let (left, right) = Node::split_by_rank(root_box.take(), rank);
        let (node, right) = Node::split_by_rank(right, 1);
        *root_box = Node::merge(left, right);
        node
    }
}

impl<K: Ord, V> OsNodeExt<K, V> for Node<K, V, OrderStatistics> {
    fn split_by_rank(
        node: Option<Box<Node<K, V, OrderStatistics>>>,
        mut rank: usize,
    ) -> (
        Option<Box<Node<K, V, OrderStatistics>>>,
        Option<Box<Node<K, V, OrderStatistics>>>,
    ) {
        Node::split_generic(node, &mut |node| {
            let order_of_left = node.left().map_or(0, |left| left.metadata().order);

            if order_of_left >= rank {
                Side::Left
            } else {
                rank -= order_of_left + 1;
                Side::Right
            }
        })
    }
}

pub type Sequence<T> = Tree<(), T, OrderStatistics>;

pub trait SequenceExt<T> {
    fn insert_at_rank(&mut self, rank: usize, value: T);
    fn push_left(&mut self, value: T);
    fn push_right(&mut self, value: T);
}
impl<T> SequenceExt<T> for Sequence<T> {
    fn insert_at_rank(&mut self, rank: usize, value: T) {
        let root_box = self.root_box_mut();
        let (left, right) = Node::split_by_rank(root_box.take(), rank);
        let root = Node::merge(left, Some(Box::new(Node::new((), value))));
        *root_box = Node::merge(root, right);
    }
    fn push_left(&mut self, value: T) {
        let root_box = self.root_box_mut();
        *root_box = Node::merge(Some(Box::new(Node::new((), value))), root_box.take());
    }
    fn push_right(&mut self, value: T) {
        let root_box = self.root_box_mut();
        *root_box = Node::merge(root_box.take(), Some(Box::new(Node::new((), value))));
    }
}
