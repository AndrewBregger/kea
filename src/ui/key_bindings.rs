use std::collections::BTreeMap;

use crate::glutin::event::{ModifiersState, VirtualKeyCode};
use crate::app::Config;
use super::edits::Operation;
use std::cmp::Ordering;

#[derive(Debug, Clone, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct Key {
    key:  VirtualKeyCode,
    mods: ModifiersState,
}

impl Key {
    pub fn new(key: VirtualKeyCode, mods: &ModifiersState) -> Self {
        Self {
            key,
            mods: mods.clone(),
        }
    }
}

// #[derive(Debug, Clone, Hash, Ord, Eq, PartialEq)]
// pub struct KeyBinding {
//     key: Key,
//     operation: Operation,
// }

// impl PartialOrd for KeyBinding {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.key.cmp(&other.key))
//     }
// }
//
// impl Ord for KeyBinding {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.key.cmp(&other.key)
//     }
// }

pub struct BindingCollection {
    bindings: BTreeMap<Key, Operation>,
}

impl BindingCollection {
    pub fn new(config: &Config) -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }

    pub fn lookup(&self, key: &Key) -> Option<&Operation> {
        self.bindings.get(key)
    }
}