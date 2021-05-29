use std::collections::{HashMap, HashSet};

pub struct DatabaseMock {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    Lists(Vec<String>),
    Sets(HashSet<String>),
}

impl DatabaseMock {
    pub fn new() -> Self {
        DatabaseMock {
            elements: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: TypeSaved) -> Option<TypeSaved> {
        self.elements.insert(key, value)
    }

    pub fn get(&mut self, key: &str) -> Option<&TypeSaved> {
        self.elements.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        self.elements.get_mut(key)
    }
}

impl Default for DatabaseMock {
    fn default() -> Self {
        Self::new()
    }
}
