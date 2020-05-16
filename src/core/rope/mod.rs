/// Rope implementation
/// Inverient:
/// 1. Lines can span multiple nodes
/// 2. however, a line will never start in the middle of a node and finish in another
/// 3. if a line starts in the middle of a node it will finish in the same node.

use std::slice::SliceIndex;

/// The encoding for how new lines are identified.
enum LineEncoding {
    CLRF,   // \r\n -> Windows
    RF      // \n -> Mac and Linux
}

/// The maximum size of a leaf string
const MAX_LEAF_SIZE: usize = 32;

/// The minimum size of a leaf string
const MIN_LEAF_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub enum RopeError {
    Empty
}

pub trait TreeData: Clone {}

#[derive(Debug, Clone)]
struct InternalNode<T> {
    /// Left child node
    left: Box<Node<T>>,
    /// Right Child node
    right: Box<Node<T>>,
    /// the weight of this node. used for indexing
    weight: usize,
    /// the weight of this node in terms of characters.
    /// if the entire text is ascii then this is redundant.
    chars: usize,
}

#[derive(Debug, Clone)]
struct LeafNode<T> {
    value: String,
    meta: Option<T>
}

#[derive(Debug, Clone)]
pub enum Node<T> {
    Internal(InternalNode<T>),
    Leaf(LeafNode<T>),
}

enum IndexMode {
    Chars,
    Bytes
}

impl<T> Node<T>
    where T: TreeData {

    fn internal(left: Node<T>, right: Node<T>) -> Self {
        let weight = left.compute_len();
        let chars = left.compute_chars();
        Self::Internal(InternalNode {
            left: Box::new(left),
            right: Box::new(right),
            weight,
            chars,
        })
    }

    fn leaf(value: &str, meta: Option<T>) -> Self {
        Self::Leaf(LeafNode::<T> {
            value: value.to_string(),
            meta
        })
    }


    fn concatenate(left: Self, right: Self) -> Self {
        Node::<T>::internal(left, right)
    }

    fn split(mut node: Self, idx: usize) -> (Self, Self) {
             
    }

    fn weight(&self) -> usize {
        match &self {
            Self::Internal(ref data) => data.weight,
            Self::Leaf(ref data) => data.value.len(),
        }
    }

    fn compute_len(&self) -> usize {
        match &self {
            Self::Internal(ref data) => {
                data.weight + data.right.compute_len()
            }
            Self::Leaf(ref data) => {
                data.value.len()
            }
        }
    }

    fn compute_chars(&self) -> usize {
        match &self {
            Self::Internal(ref data) => {
                data.chars + data.right.compute_chars()
            }
            Self::Leaf(ref data) => {
                data.value.chars().count()
            }
        }
    }

    fn index(&self, idx: usize, mode: IndexMode) -> (&LeafNode<T>, usize) {
        match mode {
            IndexMode::Chars => match &self {
                Self::Internal(ref data) => {
                    if idx < data.chars {
                        data.left.index(idx, mode)
                    }
                    else {
                        data.right.index(idx - data.chars, mode)
                    }
                }
                Self::Leaf(ref data) => {
                    (data, idx)
                }
            }
            IndexMode::Bytes => match &self {
                Self::Internal(ref data) => {
                    if idx < data.weight {
                        data.left.index(idx, mode)
                    }
                    else {
                        data.right.index(idx - data.weight, mode)
                    }

                }
                Self::Leaf(ref data) => {
                    (data, idx)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RopeMetrics {
    /// number of lines
    lines: usize,
}

impl TreeData for RopeMetrics {}

pub type Rope = Node<RopeMetrics>;

impl Rope {
    /// inserts character ch at character index idx
    pub fn insert_char(&mut self, idx: usize, ch: char) {
        unimplemented!();
    }

    /// inserts string val starting at character index idx
    pub fn insert(&mut self, idx: usize, val: &str) {
        unimplemented!()
    }

    /// remove character at idx
    pub fn remove(&mut self, idx: usize) {
        unimplemented!()
    }

    /// returns a reference to an element or slice. The indices should be by character.
    pub fn get(&self, idx: usize) -> Option<char> {
        let (leaf, idx) = self.index(idx, IndexMode::Chars);
        leaf.value.chars().nth(idx)
    }

    /// returns a reference to byte or slice of bytes. The indices should be by byte.
    pub fn get_byte(&self, idx: usize) -> Option<u8> {
        let (leaf, idx) = self.index(idx, IndexMode::Bytes);
        if idx < leaf.value.len() {
            Some(leaf.value.as_bytes()[idx])
        }
        else {
            None
        }
    }

    pub fn lines(&self) { unimplemented!() }
    pub fn num_lines(&self) { unimplemented!() }
    pub fn len(&self) -> usize { unimplemented!() }
    pub fn len_bytes(&self) -> usize { unimplemented!() }

}

struct RopeBuilder<T>
    where T: TreeData {
    root: Vec<Node<T>>
}

impl<T> RopeBuilder<T>
    where T: TreeData {
    fn new() -> Self {
        Self {
            root: Vec::new(),
        }
    }

    fn push(&mut self, mut val: &str) {
        loop {
            let slice_length = val.len();
            if slice_length == 0 {
                break;
            }

            let split_point = {
                // there is enough of the string left build a new node and enough remaining to be a minimum constraints.
                if slice_length >= MAX_LEAF_SIZE && slice_length - MAX_LEAF_SIZE >= MIN_LEAF_SIZE {
                    MAX_LEAF_SIZE
                }
                else if MAX_LEAF_SIZE > slice_length && slice_length >= MIN_LEAF_SIZE {
                    slice_length
                }
                // there is for a leaf node
                else {
                    slice_length / 2
                }
            };


            let chunk = &val[0..split_point];
            val = &val[split_point..];

            self.push_node(Node::leaf(chunk, None));
        }
    }

    fn push_node(&mut self, node: Node<T>) {
        let length = self.root.len();
        if length == 1 {
            let left = self.root.pop().unwrap();
            let new_node = Node::<T>::concatenate(left, node);
            self.root.push(new_node);
        }
        else {
            self.root.push(node);
        }
    }

    fn rope(&self) -> Result<Node<T>, RopeError> {
        match self.root.first() {
            Some(root) => Ok(root.clone()),
            None => Err(RopeError::Empty),
        }
    }
}

// impl std::str::FromStr for Rope {
//     fn from_str(val: &str) -> Self {
//     }
// }

impl std::str::FromStr for Rope {
    type Err = RopeError;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        let mut rope_builder = RopeBuilder::<RopeMetrics>::new();
        rope_builder.push(val);
        rope_builder.rope()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    /// internal tests
    #[test]
    fn concate() {
    }

    #[test]
    fn split() {

    }

    fn index() {
        let s = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam pellentesque iaculis nunc, at tristique massa.";

        let indices: Vec<usize> = vec![200, 10, 40, 100, 150];

        let rope = Rope::from_str(s).unwrap();


        for idx in indices {
            let rope_ch = rope.get(idx);
            let rope_byte = rope.get_byte(idx);

            let str_ch = s.chars().nth(idx);
            let str_byte = s.as_bytes()[idx];

            assert!(rope_ch == str_ch);
            assert!(rope_byte == Some(str_byte));
        }
    }

    /// external tests
    #[test]
    fn build() {
    }

    #[test]
    fn insert() {
    }

    #[test]
    fn remove() {
    }

    #[test]
    fn print() {
    }
}