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
    fn update<const DEDUP: bool>(node: Option<&Node<K, V, Self, DEDUP>>) -> Self;
}

impl<K: Ord, V> Metadata<K, V> for () {
    fn update<const DEDUP: bool>(_node: Option<&Node<K, V, Self, DEDUP>>) -> Self {}
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
pub struct Node<K: Ord, V, M: Metadata<K, V>, const DEDUP: bool> {
    metadata: M,

    key: K,
    value: V,
    prio: i64,
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

pub type BoxedNode<K, V, M, const DEDUP: bool> = Option<Box<Node<K, V, M, DEDUP>>>;

impl<K: Ord, V, M: Metadata<K, V>, const DEDUP: bool> Node<K, V, M, DEDUP> {
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
    pub fn left_slot_mut(&mut self) -> &mut BoxedNode<K, V, M, DEDUP> {
        &mut self.left
    }
    pub fn right(&self) -> Option<&Self> {
        self.right.as_deref()
    }
    pub fn right_mut(&mut self) -> Option<&mut Self> {
        self.right.as_deref_mut()
    }
    pub fn right_slot_mut(&mut self) -> &mut BoxedNode<K, V, M, DEDUP> {
        &mut self.right
    }

    pub fn new(key: K, value: V) -> Self {
        Self {
            metadata: M::update::<DEDUP>(None),

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

    pub fn new_boxed(key: K, value: V) -> BoxedNode<K, V, M, DEDUP> {
        Some(Box::new(Node::new(key, value)))
    }

    pub fn split_generic(
        node: BoxedNode<K, V, M, DEDUP>,
        cmp: &mut impl FnMut(&Node<K, V, M, DEDUP>) -> Side,
    ) -> (BoxedNode<K, V, M, DEDUP>, BoxedNode<K, V, M, DEDUP>) {
        if let Some(mut node) = node {
            match cmp(&node) {
                Side::Left => {
                    let (ll, lr) = Self::split_generic(node.left, cmp);
                    node.left = lr;
                    node.update_metadata();
                    (ll, Some(node))
                }
                Side::Right => {
                    let (rl, rr) = Self::split_generic(node.right, cmp);
                    node.right = rl;
                    node.update_metadata();
                    (Some(node), rr)
                }
            }
        } else {
            (None, None)
        }
    }

    pub fn split_remove_generic(
        node: BoxedNode<K, V, M, DEDUP>,
        cmp: &mut impl FnMut(&Node<K, V, M, DEDUP>) -> Ordering,
    ) -> (BoxedNode<K, V, M, DEDUP>, BoxedNode<K, V, M, DEDUP>) {
        if let Some(mut node) = node {
            match cmp(&node) {
                Ordering::Equal if DEDUP => (node.left, node.right),
                Ordering::Less | Ordering::Equal => {
                    let (ll, lr) = Self::split_remove_generic(node.left, cmp);
                    node.left = lr;
                    node.update_metadata();
                    (ll, Some(node))
                }
                Ordering::Greater => {
                    let (rl, rr) = Self::split_remove_generic(node.right, cmp);
                    node.right = rl;
                    node.update_metadata();
                    (Some(node), rr)
                }
            }
        } else {
            (None, None)
        }
    }

    pub fn merge(
        left: BoxedNode<K, V, M, DEDUP>,
        right: BoxedNode<K, V, M, DEDUP>,
    ) -> BoxedNode<K, V, M, DEDUP> {
        match (left, right) {
            (None, right) => right,
            (left, None) => left,
            (Some(mut left), Some(mut right)) => {
                if left.prio > right.prio {
                    left.right = Self::merge(left.right, Some(right));
                    left.update_metadata();
                    Some(left)
                } else {
                    right.left = Self::merge(Some(left), right.left);
                    right.update_metadata();
                    Some(right)
                }
            }
        }
    }

    /// NOTE: if cmp returns Equal and DEDUP = false, the node will be inserted on the *LEFT*. (this matters for side-effecting comparators)
    pub fn insert_generic(
        node: BoxedNode<K, V, M, DEDUP>,
        mut new_node: Box<Node<K, V, M, DEDUP>>,
        cmp: &mut impl FnMut(&Node<K, V, M, DEDUP>, &Box<Node<K, V, M, DEDUP>>) -> Ordering,
    ) -> Box<Node<K, V, M, DEDUP>> {
        if let Some(mut node) = node {
            match cmp(&new_node, &node) {
                Ordering::Equal if DEDUP => {
                    new_node.left = node.left;
                    new_node.right = node.right;
                    new_node.prio = node.prio; // might not be correct
                    new_node.update_metadata();
                    new_node
                }
                Ordering::Less | Ordering::Equal => {
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
                Ordering::Greater => {
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
        node: BoxedNode<K, V, M, DEDUP>,
        key: &Q,
    ) -> (BoxedNode<K, V, M, DEDUP>, BoxedNode<K, V, M, DEDUP>)
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Self::split_generic(node, &mut |other| Side::from_cmp(key, other.key().borrow()))
    }

    pub fn contains_key<Q>(node: Option<&Node<K, V, M, DEDUP>>, key: &Q) -> bool
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

    pub fn find<'a, Q>(
        node: Option<&'a Node<K, V, M, DEDUP>>,
        key: &Q,
    ) -> Option<&'a Node<K, V, M, DEDUP>>
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
        node: Option<&'a mut Node<K, V, M, DEDUP>>,
        key: &Q,
    ) -> Option<&'a mut Node<K, V, M, DEDUP>>
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

    pub fn find_slot_mut<'a, Q>(
        node: &'a mut BoxedNode<K, V, M, DEDUP>,
        key: &Q,
    ) -> &'a mut BoxedNode<K, V, M, DEDUP>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let comparison;
        {
            let Some(ref mut node_inner) = node else { return node; };

            comparison = key.cmp(node_inner.key().borrow());

            if comparison == Ordering::Equal {
                return node;
            }
        }

        let Some(ref mut node_inner) = node else { unreachable!() };

        match comparison {
            Ordering::Less => return Self::find_slot_mut(&mut node_inner.left, key),
            Ordering::Greater => return Self::find_slot_mut(&mut node_inner.right, key),
            _ => unreachable!(),
        }
    }

    pub fn remove(node: &mut BoxedNode<K, V, M, DEDUP>) -> Option<(K, V)> {
        if let Some(node_inner) = node.take() {
            *node = Node::merge(node_inner.left, node_inner.right).take();
            Some((node_inner.key, node_inner.value))
        } else {
            None
        }
    }

    pub fn iter(node: Option<&Node<K, V, M, DEDUP>>) -> Iter<'_, K, V, M, DEDUP> {
        if let Some(node) = node {
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
pub struct Tree<K: Ord, V, M: Metadata<K, V> = (), const DEDUP: bool = false> {
    root: BoxedNode<K, V, M, DEDUP>,
}

impl<K: Ord, V, M: Metadata<K, V>, const DEDUP: bool> Tree<K, V, M, DEDUP> {
    pub fn root(&self) -> Option<&Node<K, V, M, DEDUP>> {
        self.root.as_deref()
    }
    pub fn root_mut(&mut self) -> Option<&mut Node<K, V, M, DEDUP>> {
        self.root.as_deref_mut()
    }
    pub fn root_slot_mut(&mut self) -> &mut BoxedNode<K, V, M, DEDUP> {
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
            &mut |node, at| node.key.cmp(&at.key),
        ));
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::contains_key(self.root(), key)
    }

    pub fn iter(&self) -> Iter<'_, K, V, M, DEDUP> {
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

    pub fn find<'a, Q>(&'a self, key: &Q) -> Option<&'a Node<K, V, M, DEDUP>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find(self.root(), key)
    }
    pub fn find_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut Node<K, V, M, DEDUP>>
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

    pub fn find_slot_mut<'a, Q>(&'a mut self, key: &Q) -> &'a mut BoxedNode<K, V, M, DEDUP>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::find_slot_mut(self.root_slot_mut(), key)
    }

    pub fn remove_key<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        Node::remove(self.find_slot_mut(key))
    }
}

impl<K, V, M, const DEDUP: bool> Default for Tree<K, V, M, DEDUP>
where
    K: Ord,
    M: Metadata<K, V>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, M, Q, const DEDUP: bool> Index<&Q> for Tree<K, V, M, DEDUP>
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
impl<K, V, M, Q, const DEDUP: bool> IndexMut<&Q> for Tree<K, V, M, DEDUP>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
    M: Metadata<K, V>,
{
    fn index_mut(&mut self, index: &Q) -> &mut V {
        self.get_mut(index).unwrap()
    }
}

impl<K, V, M, const DEDUP: bool> Extend<(K, V)> for Tree<K, V, M, DEDUP>
where
    K: Ord,
    M: Metadata<K, V>,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, M, const DEDUP: bool> Extend<K> for Tree<K, (), M, DEDUP>
where
    K: Ord,
    M: Metadata<K, ()>,
{
    fn extend<T: IntoIterator<Item = K>>(&mut self, iter: T) {
        for k in iter {
            self.insert(k, ());
        }
    }
}

impl<K, V, M, const DEDUP: bool> FromIterator<(K, V)> for Tree<K, V, M, DEDUP>
where
    K: Ord,
    M: Metadata<K, V>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<K, M, const DEDUP: bool> FromIterator<K> for Tree<K, (), M, DEDUP>
where
    K: Ord,
    M: Metadata<K, ()>,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

pub struct Iter<'a, K: Ord, V, M: Metadata<K, V>, const DEDUP: bool> {
    stack: Vec<&'a Node<K, V, M, DEDUP>>,
    curr: Option<&'a Node<K, V, M, DEDUP>>,
}

impl<'a, K: Ord, V, M: Metadata<K, V>, const DEDUP: bool> Iterator for Iter<'a, K, V, M, DEDUP> {
    type Item = &'a Node<K, V, M, DEDUP>;

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

pub type Map<K, V> = Tree<K, V, (), true>;
pub type Set<T> = Tree<T, (), (), true>;
pub type Multimap<K, V> = Tree<K, V, (), false>;
pub type Multiset<T> = Tree<T, (), (), false>;
