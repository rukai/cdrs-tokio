use std::cmp::Eq;
use std::collections::HashMap;
use std::convert::Into;
use std::fmt::Debug;
use std::hash::Hash;
use std::net::IpAddr;
use std::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8};

use chrono::prelude::*;
use time::PrimitiveDateTime;
use uuid::Uuid;

use super::blob::Blob;
use super::decimal::Decimal;
use super::*;
use crate::Error;

const NULL_INT_VALUE: i32 = -1;
const NOT_SET_INT_VALUE: i32 = -2;

/// Cassandra value which could be an array of bytes, null and non-set values.
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum Value {
    Some(Vec<u8>),
    Null,
    NotSet,
}

impl Value {
    /// The factory method which creates a normal type value basing on provided bytes.
    pub fn new<B>(v: B) -> Value
    where
        B: Into<Bytes>,
    {
        Value::Some(v.into().0)
    }
}

impl Serialize for Value {
    fn serialize(&self, cursor: &mut Cursor<&mut Vec<u8>>) {
        match self {
            Value::Null => NULL_INT_VALUE.serialize(cursor),
            Value::NotSet => NOT_SET_INT_VALUE.serialize(cursor),
            Value::Some(value) => {
                let len = value.len() as CInt;
                len.serialize(cursor);
                value.serialize(cursor);
            }
        }
    }
}

impl FromCursor for Value {
    fn from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Value, Error> {
        let value_size = {
            let mut buff = [0; 4];
            cursor.read_exact(&mut buff)?;
            i32::from_be_bytes(buff)
        };
        if value_size > 0 {
            Ok(Value::Some(cursor_next_value(cursor, value_size as usize)?))
        } else if value_size == -1 {
            Ok(Value::Null)
        } else if value_size == -2 {
            Ok(Value::NotSet)
        } else {
            Err(Error::General("Could not decode query values".into()))
        }
    }
}

impl<T: Into<Bytes>> From<T> for Value {
    fn from(b: T) -> Value {
        Value::new(b.into())
    }
}

impl<T: Into<Bytes>> From<Option<T>> for Value {
    fn from(b: Option<T>) -> Value {
        match b {
            Some(b) => Value::new(b.into()),
            None => Value::Null,
        }
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct Bytes(Vec<u8>);

impl From<String> for Bytes {
    #[inline]
    fn from(value: String) -> Self {
        Bytes(value.into_bytes())
    }
}

impl From<&str> for Bytes {
    #[inline]
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<i8> for Bytes {
    #[inline]
    fn from(value: i8) -> Self {
        Bytes(vec![value as u8])
    }
}

impl From<i16> for Bytes {
    #[inline]
    fn from(value: i16) -> Self {
        Bytes(to_short(value))
    }
}

impl From<i32> for Bytes {
    #[inline]
    fn from(value: i32) -> Self {
        Bytes(to_int(value))
    }
}

impl From<i64> for Bytes {
    #[inline]
    fn from(value: i64) -> Self {
        Bytes(to_bigint(value))
    }
}

impl From<u8> for Bytes {
    #[inline]
    fn from(value: u8) -> Self {
        Bytes(vec![value])
    }
}

impl From<u16> for Bytes {
    #[inline]
    fn from(value: u16) -> Self {
        Bytes(to_u_short(value))
    }
}

impl From<u32> for Bytes {
    #[inline]
    fn from(value: u32) -> Self {
        Bytes(to_u_int(value))
    }
}

impl From<u64> for Bytes {
    #[inline]
    fn from(value: u64) -> Self {
        Bytes(to_u_big(value))
    }
}

impl From<NonZeroI8> for Bytes {
    #[inline]
    fn from(value: NonZeroI8) -> Self {
        value.get().into()
    }
}

impl From<NonZeroI16> for Bytes {
    #[inline]
    fn from(value: NonZeroI16) -> Self {
        value.get().into()
    }
}

impl From<NonZeroI32> for Bytes {
    #[inline]
    fn from(value: NonZeroI32) -> Self {
        value.get().into()
    }
}

impl From<NonZeroI64> for Bytes {
    #[inline]
    fn from(value: NonZeroI64) -> Self {
        value.get().into()
    }
}

impl From<bool> for Bytes {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Bytes(vec![1])
        } else {
            Bytes(vec![0])
        }
    }
}

impl From<Uuid> for Bytes {
    #[inline]
    fn from(value: Uuid) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<IpAddr> for Bytes {
    #[inline]
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(ip) => Bytes(ip.octets().to_vec()),
            IpAddr::V6(ip) => Bytes(ip.octets().to_vec()),
        }
    }
}

impl From<f32> for Bytes {
    #[inline]
    fn from(value: f32) -> Self {
        Bytes(to_float(value))
    }
}

impl From<f64> for Bytes {
    #[inline]
    fn from(value: f64) -> Self {
        Bytes(to_float_big(value))
    }
}

impl From<PrimitiveDateTime> for Bytes {
    #[inline]
    fn from(value: PrimitiveDateTime) -> Self {
        let ts: i64 =
            value.assume_utc().unix_timestamp() * 1_000 + value.nanosecond() as i64 / 1_000_000;
        Bytes(to_bigint(ts))
    }
}

impl From<Blob> for Bytes {
    #[inline]
    fn from(value: Blob) -> Self {
        Bytes(value.into_vec())
    }
}

impl From<Decimal> for Bytes {
    #[inline]
    fn from(value: Decimal) -> Self {
        Bytes(value.serialize_to_vec())
    }
}

impl From<NaiveDateTime> for Bytes {
    #[inline]
    fn from(value: NaiveDateTime) -> Self {
        value.timestamp_millis().into()
    }
}

impl From<DateTime<Utc>> for Bytes {
    #[inline]
    fn from(value: DateTime<Utc>) -> Self {
        value.timestamp_millis().into()
    }
}

impl<T: Into<Bytes> + Clone> From<Vec<T>> for Bytes {
    fn from(vec: Vec<T>) -> Bytes {
        let mut bytes = vec![];
        let len = vec.len() as i32;

        bytes.extend_from_slice(&len.to_be_bytes());
        bytes = vec.iter().fold(bytes, |mut acc, v| {
            let b: Bytes = v.clone().into();
            acc.append(&mut Value::new(b).serialize_to_vec());
            acc
        });
        Bytes(bytes)
    }
}

impl<K, V> From<HashMap<K, V>> for Bytes
where
    K: Into<Bytes> + Clone + Debug + Hash + Eq,
    V: Into<Bytes> + Clone + Debug,
{
    fn from(map: HashMap<K, V>) -> Bytes {
        let mut bytes: Vec<u8> = vec![];
        let len = map.len() as i32;

        bytes.extend_from_slice(&len.to_be_bytes());
        bytes = map.iter().fold(bytes, |mut acc, (k, v)| {
            let key_bytes: Bytes = k.clone().into();
            let val_bytes: Bytes = v.clone().into();
            acc.append(&mut Value::new(key_bytes).serialize_to_vec());
            acc.append(&mut Value::new(val_bytes).serialize_to_vec());
            acc
        });
        Bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_serialization() {
        assert_eq!(Value::Some(vec![1]).serialize_to_vec(), vec![0, 0, 0, 1, 1]);

        assert_eq!(
            Value::Some(vec![1, 2, 3]).serialize_to_vec(),
            vec![0, 0, 0, 3, 1, 2, 3]
        );

        assert_eq!(Value::Null.serialize_to_vec(), vec![255, 255, 255, 255]);
        assert_eq!(Value::NotSet.serialize_to_vec(), vec![255, 255, 255, 254])
    }

    #[test]
    fn test_new_value_all_types() {
        assert_eq!(
            Value::new("hello"),
            Value::Some(vec!(104, 101, 108, 108, 111))
        );
        assert_eq!(
            Value::new("hello".to_string()),
            Value::Some(vec!(104, 101, 108, 108, 111))
        );
        assert_eq!(Value::new(1_u8), Value::Some(vec!(1)));
        assert_eq!(Value::new(1_u16), Value::Some(vec!(0, 1)));
        assert_eq!(Value::new(1_u32), Value::Some(vec!(0, 0, 0, 1)));
        assert_eq!(Value::new(1_u64), Value::Some(vec!(0, 0, 0, 0, 0, 0, 0, 1)));
        assert_eq!(Value::new(1_i8), Value::Some(vec!(1)));
        assert_eq!(Value::new(1_i16), Value::Some(vec!(0, 1)));
        assert_eq!(Value::new(1_i32), Value::Some(vec!(0, 0, 0, 1)));
        assert_eq!(Value::new(1_i64), Value::Some(vec!(0, 0, 0, 0, 0, 0, 0, 1)));
        assert_eq!(Value::new(true), Value::Some(vec!(1)));
    }
}
