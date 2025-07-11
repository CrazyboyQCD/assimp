use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::utils::float_precision::Mat4;

type KeyType = u64;

// typedefs for our four configuration maps.
// We don't need more, so there is no need for a generic solution
type IntPropertyMap = BTreeMap<KeyType, i32>;
type FloatPropertyMap = BTreeMap<KeyType, f32>;
type StringPropertyMap = BTreeMap<KeyType, String>;
type MatrixPropertyMap = BTreeMap<KeyType, Mat4>;
// typedef std::map<KeyType, std::function<void *(void *)>> CallbackPropertyMap;

#[allow(unused)]
#[derive(Debug, Default)]
pub struct ExportProperties {
    int_properties: IntPropertyMap,
    float_properties: FloatPropertyMap,
    string_properties: StringPropertyMap,
    matrix_properties: MatrixPropertyMap,
    // callback_properties: CallbackPropertyMap,
}

impl ExportProperties {
    pub fn get_bool(&self, key: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        self.int_properties.get(&hasher.finish()).unwrap_or(&0) != &0
    }

    pub fn get_int(&self, key: &str) -> i32 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        *self.int_properties.get(&hasher.finish()).unwrap_or(&0)
    }
}
