use std::{cmp::Ordering, fmt::Debug};

use crate::tree::{BoxedNode, Metadata, Node, Side, Tree};

#[derive(Debug, Clone, Copy)]
pub struct OrderStatistics {
    pub order: usize,
}

impl<K: Ord, V> Metadata<K, V> for OrderStatistics {
    fn update<const DEDUP: bool>(node: Option<&Node<K, V, Self, DEDUP>>) -> Self {
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

pub trait OsTreeExt<K: Ord, V, const DEDUP: bool> {
    fn find_by_rank(&self, rank: usize) -> Option<&Node<K, V, OrderStatistics, DEDUP>>;
    fn remove_by_rank(&mut self, rank: usize) -> BoxedNode<K, V, OrderStatistics, DEDUP>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub trait OsNodeExt<K: Ord, V, const DEDUP: bool> {
    fn split_by_rank(
        node: BoxedNode<K, V, OrderStatistics, DEDUP>,
        rank: usize,
    ) -> (
        BoxedNode<K, V, OrderStatistics, DEDUP>,
        BoxedNode<K, V, OrderStatistics, DEDUP>,
    );
}

impl<K: Ord, V, const DEDUP: bool> OsTreeExt<K, V, DEDUP> for Tree<K, V, OrderStatistics, DEDUP> {
    fn find_by_rank(&self, rank: usize) -> Option<&Node<K, V, OrderStatistics, DEDUP>> {
        fn find_in_node_by_rank<K: Ord, V, const DEDUP: bool>(
            node: Option<&Node<K, V, OrderStatistics, DEDUP>>,
            rank: usize,
        ) -> Option<&Node<K, V, OrderStatistics, DEDUP>> {
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

    fn remove_by_rank(&mut self, rank: usize) -> BoxedNode<K, V, OrderStatistics, DEDUP> {
        let root_box = self.root_slot_mut();
        let (left, right) = Node::split_by_rank(root_box.take(), rank);
        let (node, right) = Node::split_by_rank(right, 1);
        *root_box = Node::merge(left, right);
        node
    }

    fn len(&self) -> usize {
        self.root().map_or(0, |node| node.metadata().order)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Ord, V, const DEDUP: bool> OsNodeExt<K, V, DEDUP> for Node<K, V, OrderStatistics, DEDUP> {
    fn split_by_rank(
        node: BoxedNode<K, V, OrderStatistics, DEDUP>,
        mut rank: usize,
    ) -> (
        BoxedNode<K, V, OrderStatistics, DEDUP>,
        BoxedNode<K, V, OrderStatistics, DEDUP>,
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

pub type Sequence<T> = Tree<(), T, OrderStatistics, false>;

pub trait SequenceExt<T> {
    fn insert_at_rank(&mut self, rank: usize, value: T);
    fn push_left(&mut self, value: T);
    fn push_right(&mut self, value: T);
}
impl<T> SequenceExt<T> for Sequence<T> {
    fn insert_at_rank(&mut self, mut rank: usize, value: T) {
        let root_box = self.root_slot_mut();
        let node = Box::new(Node::new((), value));
        *root_box = Some(Node::insert_generic(
            root_box.take(),
            node,
            &mut |_node, against| {
                let order_of_left = against.left().map_or(0, |node| node.metadata().order);
                match rank.cmp(&order_of_left) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => {
                        rank -= order_of_left + 1;
                        Ordering::Greater
                    }
                }
            },
        ));
    }
    fn push_left(&mut self, value: T) {
        let root_box = self.root_slot_mut();
        let node = Node::new_boxed((), value);
        *root_box = Node::merge(node, root_box.take());
    }
    fn push_right(&mut self, value: T) {
        let root_box = self.root_slot_mut();
        let node = Node::new_boxed((), value);
        *root_box = Node::merge(root_box.take(), node);
    }
}
