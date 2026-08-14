//! A generic protobuf wire-format decoder ported from GarupaSpeedTracker's
//! `GarupaParser`. It reads tag/wire-type/length-value fields from raw binary
//! and maps them to JSON values according to a runtime [`Schema`] descriptor,
//! so no precompiled `.proto` files are needed.
//!
//! Behavior mirrors the reference implementation:
//! - Wire types are validated against the schema; mismatches are skipped.
//! - Repeated fields produce arrays; non-repeated fields take the last valid
//!   occurrence, which is robust to trailing garbage.

use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde_json::{Map, Value};

use super::schema::{ProtoType, Schema};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireType {
    Varint,
    Fixed64,
    LengthDelimited,
    Fixed32,
}

#[derive(Debug, Clone)]
enum RawData {
    Varint(u64),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
struct RawField {
    field: u32,
    wire_type: WireType,
    data: RawData,
}

#[derive(Debug)]
pub struct ProtoError(pub String);

impl std::fmt::Display for ProtoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ProtoError {}

/// Reads a base-128 varint from `buf` starting at `offset`.
/// Returns the decoded value and the new offset, or `None` on truncation or overflow.
fn read_varint(buf: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    let mut cursor = offset;

    while cursor < buf.len() {
        let byte = buf[cursor];
        cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some((value, cursor));
        }
        shift += 7;
        if shift > 56 {
            return None;
        }
    }

    None
}

/// Parses the whole buffer into a flat list of raw protobuf fields.
/// Malformed tail bytes are silently ignored, matching the reference.
fn parse_raw_fields(buf: &[u8]) -> Vec<RawField> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset < buf.len() {
        let (key, next) = match read_varint(buf, offset) {
            Some(v) => v,
            None => break,
        };
        offset = next;
        if key == 0 {
            break;
        }

        let field = (key >> 3) as u32;
        let wire_type = key & 0x07;
        if field == 0 {
            break;
        }

        match wire_type {
            0 => {
                let (value, new_offset) = match read_varint(buf, offset) {
                    Some(v) => v,
                    None => break,
                };
                offset = new_offset;
                results.push(RawField { field, wire_type: WireType::Varint, data: RawData::Varint(value) });
            }
            2 => {
                let (len, new_offset) = match read_varint(buf, offset) {
                    Some(v) => v,
                    None => break,
                };
                offset = new_offset;
                let len = len as usize;
                if offset + len > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + len].to_vec();
                offset += len;
                results.push(RawField { field, wire_type: WireType::LengthDelimited, data: RawData::Bytes(inner) });
            }
            1 => {
                if offset + 8 > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + 8].to_vec();
                offset += 8;
                results.push(RawField { field, wire_type: WireType::Fixed64, data: RawData::Bytes(inner) });
            }
            5 => {
                if offset + 4 > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + 4].to_vec();
                offset += 4;
                results.push(RawField { field, wire_type: WireType::Fixed32, data: RawData::Bytes(inner) });
            }
            _ => break,
        }
    }

    results
}

/// Decodes a protobuf buffer into a JSON object according to `schema`.
pub fn decode(buf: &[u8], schema: &Schema) -> Result<Value, ProtoError> {
    let raw_fields = parse_raw_fields(buf);

    let mut groups: HashMap<u32, Vec<RawField>> = HashMap::new();
    for field in raw_fields {
        groups.entry(field.field).or_default().push(field);
    }

    let mut result = Map::new();

    for (tag, field_def) in schema.fields {
        let items = match groups.get(tag) {
            Some(items) if !items.is_empty() => items,
            _ => continue,
        };

        let parse = |item: &RawField| -> Option<Value> {
            let wt = item.wire_type;
            match field_def.ty {
                ProtoType::Int | ProtoType::Long => {
                    if wt != WireType::Varint {
                        return None;
                    }
                    match &item.data {
                        RawData::Varint(v) => Some(Value::from(*v as i64)),
                        _ => None,
                    }
                }
                ProtoType::Bool => {
                    if wt != WireType::Varint {
                        return None;
                    }
                    match &item.data {
                        RawData::Varint(v) => Some(Value::Bool(*v == 1)),
                        _ => None,
                    }
                }
                ProtoType::String => {
                    if wt != WireType::LengthDelimited {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) => Some(Value::String(String::from_utf8_lossy(b).into_owned())),
                        _ => None,
                    }
                }
                ProtoType::Bytes => {
                    if wt != WireType::LengthDelimited {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) => Some(Value::String(BASE64.encode(b))),
                        _ => None,
                    }
                }
                ProtoType::Message(sub) => {
                    if wt != WireType::LengthDelimited {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) => decode(b, sub).ok(),
                        _ => None,
                    }
                }
                ProtoType::Double => {
                    if wt != WireType::Fixed64 {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) if b.len() == 8 => {
                            let arr: [u8; 8] = b.clone().try_into().ok()?;
                            Some(Value::from(f64::from_le_bytes(arr)))
                        }
                        _ => None,
                    }
                }
                ProtoType::Float => {
                    if wt != WireType::Fixed32 {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) if b.len() == 4 => {
                            let arr: [u8; 4] = b.clone().try_into().ok()?;
                            Some(Value::from(f32::from_le_bytes(arr)))
                        }
                        _ => None,
                    }
                }
            }
        };

        if field_def.repeated {
            let arr: Vec<Value> = items.iter().filter_map(&parse).collect();
            result.insert(field_def.name.to_string(), Value::Array(arr));
        } else {
            for item in items.iter().rev() {
                if let Some(v) = parse(item) {
                    result.insert(field_def.name.to_string(), v);
                    break;
                }
            }
        }
    }

    Ok(Value::Object(result))
}

/// Test-only helper that dumps every raw protobuf field for schema discovery.
/// Strings are detected when the bytes are valid printable UTF-8; otherwise
/// length-delimited data is re-parsed recursively as a nested message.
#[cfg(test)]
pub fn dump_raw(buf: &[u8]) -> Value {
    use serde_json::json;
    fn dump(buf: &[u8]) -> Vec<Value> {
        parse_raw_fields(buf)
            .iter()
            .map(|f| {
                let wire = match f.wire_type {
                    WireType::Varint => "varint",
                    WireType::Fixed64 => "fixed64",
                    WireType::LengthDelimited => "len",
                    WireType::Fixed32 => "fixed32",
                };
                match &f.data {
                    RawData::Varint(v) => json!({ "field": f.field, "wire": wire, "value": v }),
                    RawData::Bytes(b) => {
                        if !b.is_empty() {
                            if let Ok(s) = std::str::from_utf8(b) {
                                if !s.chars().any(|c| c.is_control()) {
                                    return json!({ "field": f.field, "wire": wire, "string": s });
                                }
                            }
                        }
                        let nested = dump(b);
                        if nested.is_empty() {
                            json!({ "field": f.field, "wire": wire, "bytes": b.len() })
                        } else {
                            json!({ "field": f.field, "wire": wire, "message": nested })
                        }
                    }
                }
            })
            .collect()
    }
    Value::Array(dump(buf))
}

/// Test-only helper that dumps the first entry of a field-1 list wrapper.
/// Used for schema discovery on large master tables.
#[cfg(test)]
pub fn first_entry_dump(buf: &[u8]) -> Option<Value> {
    let raw = parse_raw_fields(buf);
    let item = raw.iter().find(|f| f.field == 1 && matches!(f.data, RawData::Bytes(_)))?;
    if let RawData::Bytes(b) = &item.data {
        Some(dump_raw(b))
    } else {
        None
    }
}

/// Test-only helper that collects the union of field numbers and wire types
/// across the first `sample` entries of a field-1 list wrapper.
#[cfg(test)]
pub fn top_field_union(buf: &[u8], sample: usize) -> Value {
    use serde_json::json;
    let raw = parse_raw_fields(buf);
    let mut counts: std::collections::BTreeMap<u32, std::collections::BTreeMap<&'static str, usize>> =
        std::collections::BTreeMap::new();
    for (records, f) in raw.iter().filter(|f| f.field == 1 && matches!(f.data, RawData::Bytes(_))).enumerate() {
        if records >= sample {
            break;
        }
        if let RawData::Bytes(b) = &f.data {
            for nf in parse_raw_fields(b) {
                let wt = match nf.wire_type {
                    WireType::Varint => "int",
                    WireType::Fixed64 => "f64",
                    WireType::LengthDelimited => "len",
                    WireType::Fixed32 => "f32",
                };
                *counts.entry(nf.field).or_default().entry(wt).or_default() += 1;
            }
        }
    }
    counts
        .iter()
        .map(|(field, types)| json!({ "field": field, "types": types }))
        .collect()
}

/// Test-only helper that dumps the first `limit` list-wrapper entries containing
/// any of the given field numbers. Used to identify rare fields that a schema is
/// missing; the raw bytes of each matching entry are decoded recursively.
#[cfg(test)]
pub fn entries_containing_fields(buf: &[u8], fields: &[u32], limit: usize) -> Vec<Value> {
    let mut out = Vec::new();
    for item in parse_raw_fields(buf)
        .iter()
        .filter(|f| f.field == 1 && matches!(f.data, RawData::Bytes(_)))
    {
        if out.len() >= limit {
            break;
        }
        if let RawData::Bytes(b) = &item.data {
            if parse_raw_fields(b).iter().any(|nf| fields.contains(&nf.field)) {
                out.push(dump_raw(b));
            }
        }
    }
    out
}

/// Counts the raw protobuf fields in a buffer. Kept as a discovery probe helper
/// for live endpoint scans; no handler relies on it now that list endpoints
/// pass through whatever the game returns.
#[allow(dead_code)]
pub fn raw_field_count(buf: &[u8]) -> usize {
    parse_raw_fields(buf).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::garupa_schema::RANKING_USER_LIST_SCHEMA;
    /// Test helper that appends a base-128 varint to a buffer.
    fn push_varint(buf: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    #[test]
    fn reads_varint() {
        assert_eq!(read_varint(&[0x2a], 0), Some((42, 1)));
        assert_eq!(read_varint(&[0xac, 0x02], 0), Some((300, 2)));
        assert_eq!(read_varint(&[0x80, 0x01], 0), Some((128, 2)));
    }

    #[test]
    fn decodes_ranking_user_list() {
        // One ranking user:
        //   name    field 1, wire 2, len 6, "tester"
        //   userId  field 7, wire 0, 42
        //   point   field 6, wire 0, 100
        //   rank    field 5, wire 0, 2
        let mut user = Vec::new();
        user.extend_from_slice(&[0x0a, 6]);
        user.extend_from_slice(b"tester");
        user.extend_from_slice(&[0x38, 42]);
        user.extend_from_slice(&[0x30, 100]);
        user.extend_from_slice(&[0x28, 2]);

        // List wrapper: field 1 holds the entries with wire type 2.
        let mut list = Vec::new();
        list.push(0x0a);
        push_varint(&mut list, user.len() as u64);
        list.extend_from_slice(&user);

        let value = decode(&list, &RANKING_USER_LIST_SCHEMA).unwrap();
        let entries = value["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        let u = &entries[0];
        assert_eq!(u["name"], "tester");
        assert_eq!(u["userId"], 42);
        assert_eq!(u["point"], 100);
        assert_eq!(u["rank"], 2);
    }

    #[test]
    fn ignores_wrong_wire_types() {
        // Send userId as a length-delimited value even though it should be a varint.
        let buf = [0x3a, 1, 0xff]; // field 7 wire 2 len 1 -> should be skipped
        let value = decode(&buf, &RANKING_USER_LIST_SCHEMA).unwrap();
        assert!(value.get("entries").is_none() || value["entries"].as_array().unwrap().is_empty());
    }

    #[test]
    fn counts_raw_fields() {
        assert_eq!(raw_field_count(&[]), 0);
        assert_eq!(raw_field_count(&[0x0a, 0x01, 0x00]), 1);
    }
}
