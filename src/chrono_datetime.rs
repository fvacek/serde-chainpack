use chrono::{DateTime, FixedOffset};
use serde::{de, Deserializer, Serializer};
use std::fmt;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub struct ChainPackDateTime(pub DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for ChainPackDateTime {
    fn from(dt: DateTime<FixedOffset>) -> Self {
        ChainPackDateTime(dt)
    }
}

impl From<ChainPackDateTime> for DateTime<FixedOffset> {
    fn from(val: ChainPackDateTime) -> Self {
        val.0
    }
}

impl Serialize for ChainPackDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let dt = &self.0;
        let epoch_msec = dt.timestamp_millis();
        let utc_offset = dt.offset().local_minus_utc();

        let mut writer = Vec::new();
        writer.write_u8(crate::types::CP_DATETIME).map_err(serde::ser::Error::custom)?;
        writer.write_i64::<LittleEndian>(epoch_msec).map_err(serde::ser::Error::custom)?;
        writer.write_i32::<LittleEndian>(utc_offset).map_err(serde::ser::Error::custom)?;

        serializer.serialize_bytes(&writer)
    }
}

impl<'de> Deserialize<'de> for ChainPackDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(DateTimeVisitor).map(ChainPackDateTime)
    }
}

struct DateTimeVisitor;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<FixedOffset>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ChainPack DateTime")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        let mut reader = std::io::Cursor::new(v);
        let type_byte = reader.read_u8().map_err(E::custom)?;
        if type_byte != crate::types::CP_DATETIME {
            return Err(E::custom("Expected CP_DATETIME type byte"));
        }

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
