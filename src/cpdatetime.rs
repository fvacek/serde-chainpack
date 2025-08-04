use chrono::{DateTime, FixedOffset};
use serde::{de, Deserializer, Serializer};
use std::fmt;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use crate::types;

#[derive(Debug, PartialEq)]
pub struct CPDateTime(pub DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for CPDateTime {
    fn from(dt: DateTime<FixedOffset>) -> Self {
        CPDateTime(dt)
    }
}

impl From<CPDateTime> for DateTime<FixedOffset> {
    fn from(val: CPDateTime) -> Self {
        val.0
    }
}

use serde_bytes::Bytes;

impl Serialize for CPDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut writer = Vec::new();
        writer.write_i64::<LittleEndian>(self.0.timestamp_millis()).map_err(serde::ser::Error::custom)?;
        writer.write_i32::<LittleEndian>(self.0.offset().local_minus_utc()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_newtype_struct(types::CP_DATETIME_STRUCT, Bytes::new(&writer))
    }
}

impl<'de> Deserialize<'de> for CPDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(types::CP_DATETIME_STRUCT, DateTimeVisitor).map(CPDateTime)
    }
}

struct DateTimeVisitor;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<FixedOffset>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ChainPack DateTime")
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(self)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() != 12 {
            return Err(E::invalid_length(v.len(), &"12 bytes"));
        }
        let mut reader = std::io::Cursor::new(v);
        let epoch_msec = reader.read_i64::<LittleEndian>().map_err(E::custom)?;
        let utc_offset = reader.read_i32::<LittleEndian>().map_err(E::custom)?;

        let dt = DateTime::from_timestamp_millis(epoch_msec)
            .ok_or_else(|| E::custom(format!("invalid timestamp milliseconds: {}", epoch_msec)))?;

        let offset = FixedOffset::east_opt(utc_offset)
            .ok_or_else(|| E::custom(format!("invalid timezone offset: {}", utc_offset)))?;

        Ok(dt.with_timezone(&offset))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(&v)
    }
}
