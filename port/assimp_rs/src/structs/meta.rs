use indexmap::IndexMap;

use crate::{AiReal, utils::float_precision::Vec3};

#[derive(Debug, Clone)]
pub enum MetadataEntry {
    Bool(bool),
    Int32(i32),
    UInt64(u64),
    Float(AiReal),
    String(Box<str>),
    Vector3(Vec3),
    Metadata(Box<Metadata>),
    Int64(i64),
    UInt32(u32),
    MetaMax(()),
}

impl PartialEq for MetadataEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MetadataEntry::Bool(a), MetadataEntry::Bool(b)) => a == b,
            (MetadataEntry::Int32(a), MetadataEntry::Int32(b)) => a == b,
            (MetadataEntry::UInt64(a), MetadataEntry::UInt64(b)) => a == b,
            (MetadataEntry::Float(a), MetadataEntry::Float(b)) => a == b,
            (MetadataEntry::String(a), MetadataEntry::String(b)) => a == b,
            (MetadataEntry::Vector3(a), MetadataEntry::Vector3(b)) => a == b,
            (MetadataEntry::Metadata(a), MetadataEntry::Metadata(b)) => a == b,
            (MetadataEntry::Int64(a), MetadataEntry::Int64(b)) => a == b,
            (MetadataEntry::UInt32(a), MetadataEntry::UInt32(b)) => a == b,
            (MetadataEntry::MetaMax(()), MetadataEntry::MetaMax(())) => true,
            _ => false,
        }
    }
}

impl Default for MetadataEntry {
    fn default() -> Self {
        Self::MetaMax(())
    }
}

pub type Metadata = IndexMap<String, MetadataEntry>;
