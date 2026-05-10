use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use serde::{Serialize, ser::SerializeSeq};

#[derive(Default)]
pub struct HashMapVector<K, V>(HashMap<K, V>);

impl<K, V> Deref for HashMapVector<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for HashMapVector<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V: Serialize> Serialize for HashMapVector<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let values = self.0.values();
        let mut state = serializer.serialize_seq(Some(values.len()))?;

        for (_, element) in &self.0 {
            state.serialize_element(element)?;
        }

        state.end()
    }
}


#[derive(Default, Clone)]
pub struct HashSetVector<V>(HashSet<V>);

impl<V> Deref for HashSetVector<V> {
    type Target = HashSet<V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for HashSetVector<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V: Serialize> Serialize for HashSetVector<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.0.len()))?;

        for element in &self.0 {
            state.serialize_element(element)?;
        }

        state.end()
    }
}
