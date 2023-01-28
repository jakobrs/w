use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::Debug,
    ops::{Index, IndexMut},
};

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

impl Side {
    pub fn from_cmp<K: Ord>(lhs: K, rhs: K) -> Self {
        if lhs < rhs {
            Side::Left
        } else {
            Side::Right
        }
    }
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

pub type BoxedNode<K, V, M> = Option<Box<Node<K, V, M>>>;

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

    fn update_metadata(&mut self) {
        self.metadata = M::update(Some(self));
    }

    pub fn new_boxed(key: K, value: V) -> BoxedNode<K, V, M> {
        Some(Box::new(Node::new(key, value)))
    }

    pub fn split_generic(
        node: BoxedNode<K, V, M>,
        cmp: &mut impl FnMut(&Node<K, V, M>) -> Side,
    ) -> (BoxedNode<K, V, M>, BoxedNode<K, V, M>) {
        if let Some(mut node) = node {
            match cmp(&node) {
                Side::Left => {
                    let (ll, lr) = Self::split_generic(node.left, cmp);
                    node.left = lr;
                    node.metadata = M::update(Some(&node));
                    (ll, Some(node))
                }
                Side::Right => {
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

    pub fn merge(left: BoxedNode<K, V, M>, right: BoxedNode<K, V, M>) -> BoxedNode<K, V, M> {
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

    pub fn insert_generic(
        node: BoxedNode<K, V, M>,
        new_node: Box<Node<K, V, M>>,
        cmp: &mut impl FnMut(&Node<K, V, M>, &Box<Node<K, V, M>>) -> Side,
    ) -> Box<Node<K, V, M>> {
        if let Some(mut node) = node {
            match cmp(&new_node, &node) {
                Side::Left => {
                    let mut subtree = Self::insert_generic(node.left, new_node, cmp);
                    if subtree.prio > node.prio {
                        node.left = subtree.right;
                        node.update_metadata();
                        subtree.right = Some(node);
                        subtree.update_metadata();
                        subtree
                    } else {
                        node.left = Some(subtree);
                        node.update_metadata();
                        node
                    }
                }
                Side::Right => {
                    let mut subtree = Self::insert_generic(node.right, new_node, cmp);
                    if subtree.prio > node.prio {
                        node.right = subtree.left;
                        node.update_metadata();
                        subtree.left = Some(node);
                        subtree.update_metadata();
                        subtree
                    } else {
                        node.right = Some(subtree);
                        node.update_metadata();
                        node
                    }
                }
            }
        } else {
            new_node
        }
    }

    pub fn split_before<Q>(
        node: BoxedNode<K, V, M>,
        key: &Q,
    ) -> (BoxedNode<K, V, M>, BoxedNode<K, V, M>)
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Self::split_generic(node, &mut |other| Side::from_cmp(key, other.key().borrow()))
    }

    pub fn contains_key<Q>(node: Option<&Node<K, V, M>>, key: &Q) -> bool
    where
        K: Borrow<Q>,
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

    pub fn find<'a, Q>(node: Option<&'a Node<K, V, M>>, key: &Q) -> Option<&'a Node<K, V, M>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        node.and_then(|node| match key.cmp(node.key().borrow()) {
            Ordering::Less => Self::find(node.left(), key),
            Ordering::Equal => Some(node),
            Ordering::Greater => Self::find(node.right(), key),
        })
    }

    pub fn find_mut<'a, Q>(
        node: Option<&'a mut Node<K, V, M>>,
        key: &Q,
    ) -> Option<&'a mut Node<K, V, M>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        node.and_then(|node| match key.cmp(node.key().borrow()) {
            Ordering::Less => Self::find_mut(node.left_mut(), key),
            Ordering::Equal => Some(node),
            Ordering::Greater => Self::find_mut(node.right_mut(), key),
        })
    }

    pub fn iter(node: Option<&Node<K, V, M>>) -> Iter<'_, K, V, M> {
        if let Some(ref node) = node {
            Iter {
                stack: vec![],
                curr: Some(node),
            }
        } else {
            Iter {
                stack: vec![],
                curr: None,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tree<K: Ord, V, M: Metadata<K, V> = ()> {
    root: BoxedNode<K, V, M>,
}

impl<K: Ord, V, M: Metadata<K, V>> Tree<K, V, M> {
    pub fn root(&self) -> Option<&Node<K, V, M>> {
        self.root.as_deref()
    }
    pub fn root_mut(&mut self) -> Option<&mut Node<K, V, M>> {
        self.root.as_deref_mut()
    }
    pub fn root_box_mut(&mut self) -> &mut BoxedNode<K, V, M> {
        &mut self.root
    }

    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let node = Box::new(Node::new(key, value));
        self.root = Some(Node::insert_generic(
            self.root.take(),
            node,
            &mut |node, at| Side::from_cmp(&node.key, &at.key),
        ));
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
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

    pub fn find<'a, Q>(&'a self, key: &Q) -> Option<&'a Node<K, V, M>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find(self.root(), key)
    }
    pub fn find_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut Node<K, V, M>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find_mut(self.root_mut(), key)
    }

    pub fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find(self.root(), key).map(|node| node.value())
    }
    pub fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find_mut(self.root_mut(), key).map(|node| node.value_mut())
    }
}

impl<K, V, M, Q> Index<&Q> for Tree<K, V, M>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
    M: Metadata<K, V>,
{
    type Output = V;

    fn index(&self, index: &Q) -> &V {
        self.get(index).unwrap()
    }
}
impl<K, V, M, Q> IndexMut<&Q> for Tree<K, V, M>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
    M: Metadata<K, V>,
{
    fn index_mut(&mut self, index: &Q) -> &mut V {
        self.get_mut(index).unwrap()
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
