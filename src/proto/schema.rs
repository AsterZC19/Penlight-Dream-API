//! Runtime protobuf schema descriptors ported from GarupaSpeedTracker's
//! `SchemaDefinition`. Schemas map protobuf field numbers to names and types so
//! the generic wire-format decoder can turn Garupa responses into JSON without
//! precompiled `.proto` files.

/// Expected wire type for a schema field, used for lenient validation.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // Double/Float/Bytes are part of the decoder's capability; no current schema uses them
pub enum ProtoType {
    /// varint → integer, also used for 64-bit longs
    Int,
    /// varint → integer
    Long,
    /// length-delimited → UTF-8 string
    String,
    /// varint → boolean
    Bool,
    /// 64-bit fixed → f64
    Double,
    /// 32-bit fixed → f32
    Float,
    /// length-delimited → raw bytes, base64 in JSON output
    Bytes,
    /// length-delimited → nested message
    Message(&'static Schema),
}

/// A single field within a [`Schema`].
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub ty: ProtoType,
    pub repeated: bool,
}

/// A protobuf message descriptor built from a static field table.
#[derive(Clone, Copy, Debug)]
pub struct Schema {
    pub fields: &'static [(u32, Field)],
}

/// Convenience constructor used inside the schema tables.
pub const fn field(name: &'static str, ty: ProtoType, repeated: bool) -> Field {
    Field { name, ty, repeated }
}
