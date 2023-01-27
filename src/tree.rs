use std::{borrow::Borrow, cmp::Ordering, fmt::Debug};

use rand::Rng;

pub trait Metadata<K: Ord, V>
where
    Self: Sized,
{
    fn update(node: Option<&Node<K, V, Self>>) -> Self;
}

impl<K: Ord, V> Metadata<K, V> for () {
    fn update(_node: Option<&Node<K, V, Self>>) -> () {
        ()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Node<K: Ord, V, M: Metadata<K, V>> {
    metadata: M,

    key: K,
    value: V,
    prio: i64,
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

impl<K: Ord, V, M: Metadata<K, V>> Node<K, V, M> {
    pub fn key(&self) -> &K {
        &self.key
    }
    pub fn value(&self) -> &V {
        &self.value
    }
    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    pub fn metadata(&self) -> &M {
        &self.metadata
    }

    pub fn left(&self) -> Option<&Self> {
        self.left.as_deref()
    }
    pub fn left_mut(&mut self) -> Option<&mut Self> {
        self.left.as_deref_mut()
    }
    pub fn right(&self) -> Option<&Self> {
        self.right.as_deref()
    }
    pub fn right_mut(&mut self) -> Option<&mut Self> {
        self.right.as_deref_mut()
    }

    pub fn new(key: K, value: V) -> Self {
        Self {
            metadata: M::update(None),

            key,
            value,
            prio: rand::thread_rng().gen::<i64>(),
            left: None,
            right: None,
        }
    }

    pub fn split_generic(
        node: Option<Box<Node<K, V, M>>>,
        cmp: &mut impl FnMut(&Node<K, V, M>) -> Side,
    ) -> (Option<Box<Node<K, V, M>>>, Option<Box<Node<K, V, M>>>) {
        if let Some(mut node) = node {
            match cmp(&node) {
                Side::Right => {
                    let (ll, lr) = Self::split_generic(node.left, cmp);
                    node.left = lr;
                    node.metadata = M::update(Some(&node));
                    (ll, Some(node))
                }
                Side::Left => {
                    let (rl, rr) = Self::split_generic(node.right, cmp);
                    node.right = rl;
                    node.metadata = M::update(Some(&node));
                    (Some(node), rr)
                }
            }
        } else {
            (None, None)
        }
    }

    pub fn merge(
        left: Option<Box<Node<K, V, M>>>,
        right: Option<Box<Node<K, V, M>>>,
    ) -> Option<Box<Node<K, V, M>>> {
        match (left, right) {
            (None, right) => right,
            (left, None) => left,
            (Some(mut left), Some(mut right)) => {
                if left.prio > right.prio {
                    left.right = Self::merge(left.right, Some(right));
                    left.metadata = M::update(Some(&left));
                    Some(left)
                } else {
                    right.left = Self::merge(Some(left), right.left);
                    right.metadata = M::update(Some(&right));
                    Some(right)
                }
            }
        }
    }

    pub fn split_before<Q>(
        node: Option<Box<Node<K, V, M>>>,
        key: &Q,
    ) -> (Option<Box<Node<K, V, M>>>, Option<Box<Node<K, V, M>>>)
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Self::split_generic(node, &mut |other| {
            if other.key().borrow() < key {
                Side::Left
            } else {
                Side::Right
            }
        })
    }

    pub fn contains_key<Q>(node: Option<&Node<K, V, M>>, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        if let Some(node) = node {
            match key.cmp(node.key().borrow()) {
                Ordering::Less => Self::contains_key(node.left(), key),
                Ordering::Equal => true,
                Ordering::Greater => Self::contains_key(node.right(), key),
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tree<K: Ord, V, M: Metadata<K, V> = ()> {
    root: Option<Box<Node<K, V, M>>>,
}

impl<K: Ord, V, M: Metadata<K, V>> Tree<K, V, M> {
    pub fn root(&self) -> Option<&Node<K, V, M>> {
        self.root.as_deref()
    }
    pub fn root_box_mut(&mut self) -> &mut Option<Box<Node<K, V, M>>> {
        &mut self.root
    }

    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let root = self.root.take();
        let (left, right) = Node::split_before(root, &key);
        let node = Node::new(key, value);
        let root = Node::merge(left, Some(Box::new(node)));
        self.root = Node::merge(root, right);
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Node::contains_key(self.root(), key)
    }

    pub fn iter(&self) -> Iter<'_, K, V, M> {
        if let Some(ref root) = self.root {
            Iter {
                stack: vec![],
                curr: Some(root),
            }
        } else {
            Iter {
                stack: vec![],
                curr: None,
            }
        }
    }
}

pub struct Iter<'a, K: Ord, V, M: Metadata<K, V>> {
    stack: Vec<&'a Node<K, V, M>>,
    curr: Option<&'a Node<K, V, M>>,
}

impl<'a, K: Ord, V, M: Metadata<K, V>> Iterator for Iter<'a, K, V, M> {
    type Item = &'a Node<K, V, M>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(curr) = self.curr.take() {
            self.stack.push(curr);
            self.curr = curr.left.as_deref();
        }

        let val = self.stack.pop();
        self.curr = if let Some(val) = val {
            val.right.as_deref()
        } else {
            None
        };

        val
    }
}

pub type Map<K, V> = Tree<K, V>;
pub type Set<T> = Tree<T, ()>;
