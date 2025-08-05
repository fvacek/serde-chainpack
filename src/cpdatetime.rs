use chrono::{DateTime, FixedOffset};
use serde::{de, Deserializer, Serializer};
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::CPDateTime;

pub(crate) const CP_DATETIME_NEWTYPE_STRUCT: &str = "CPDateTime";

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

const SHV_EPOCH_MSEC: i64 = 1517529600000;

impl Serialize for CPDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut val = self.0.timestamp_millis() - SHV_EPOCH_MSEC;
        let mut has_tz = false;
        let mut no_msec = false;

        if self.0.timestamp_subsec_millis() == 0 {
            val /= 1000;
            no_msec = true;
        }

        let offset_minutes = self.0.offset().local_minus_utc() / 60;
        if offset_minutes != 0 {
            val <<= 7;
            let tz_offset = (offset_minutes / 15) as i64;
            val |= tz_offset & 0x7f;
            has_tz = true;
        }

        val <<= 2;
        if has_tz {
            val |= 1;
        }
        if no_msec {
            val |= 2;
        }
        serializer.serialize_newtype_struct(CP_DATETIME_NEWTYPE_STRUCT, &val)
    }
}

impl<'de> Deserialize<'de> for CPDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(CP_DATETIME_NEWTYPE_STRUCT, DateTimeVisitor).map(CPDateTime)
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
        deserializer.deserialize_i64(self)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error
    {
        let has_tz = v & 1 != 0;
        let no_msec = v & 2 != 0;
        let mut val = v >> 2;

        let offset_secs = if has_tz {
            let tz_offset = (val & 0x7f) as i32;
            val >>= 7;
            if tz_offset & 0x40 != 0 {
                (tz_offset | !0x7f) * 15 * 60
            } else {
                tz_offset * 15 * 60
            }
        } else {
            0
        };

        let msecs = if no_msec {
            val * 1000
        } else {
            val
        };

        let final_msecs = msecs + SHV_EPOCH_MSEC;
        let naive_dt = DateTime::from_timestamp(final_msecs / 1000, (final_msecs.rem_euclid(1000) * 1_000_000) as u32)
            .ok_or_else(|| de::Error::custom(format!("invalid timestamp milliseconds: {}", final_msecs)))?;
        let offset = FixedOffset::east_opt(offset_secs)
            .ok_or_else(|| de::Error::custom(format!("invalid timezone offset: {}", offset_secs)))?;
        Ok(naive_dt.with_timezone(&offset))
    }
}
