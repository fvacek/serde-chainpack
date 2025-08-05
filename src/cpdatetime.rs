use chrono::{DateTime, FixedOffset};
use std::fmt;
use serde::{Deserialize, Serialize};
use serde::{de, Deserializer, Serializer};

#[derive(Debug, PartialEq)]
pub struct CPDateTime(pub DateTime<FixedOffset>);
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde::{Deserialize, Serialize};
    use crate::{de::from_slice, ser::tests::to_vec, types::CP_DATETIME};
    use super::CPDateTime;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Event {
        timestamp: CPDateTime,
    }

    #[test]
    fn test_datetime_serde() {
        let test_cases = vec![
            ("1970-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110001, 0b10000001, 0b01101001, 0b11001110, 0b10100111, 0b11111110]),
            ("2018-02-02T00:00:00.001+00:00", vec![CP_DATETIME, 0b00000100]),
            ("2018-02-02T01:00:00.001+01:00", vec![CP_DATETIME, 0b10000010, 0b00010001]),
            ("2018-12-02T00:00:00+00:00", vec![CP_DATETIME, 0b11100110, 0b00111101, 0b11011010, 0b00000010]),
            ("2018-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11101000, 0b10101000, 0b10111111, 0b11111110]),
            ("2019-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11100110, 0b11011100, 0b00001110, 0b00000010]),
            ("2020-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110000, 0b00001110, 0b01100000, 0b11011100, 0b00000010]),
            ("2021-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110000, 0b00010101, 0b11101010, 0b11110000, 0b00000010]),
            ("2031-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110000, 0b01100001, 0b00100101, 0b10001000, 0b00000010]),
            ("2041-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110001, 0b00000000, 0b10101100, 0b01100101, 0b01100110, 0b00000010]),
            ("2041-03-04T00:00:00-10:15", vec![CP_DATETIME, 0b11110001, 0b01010110, 0b11010111, 0b01001101, 0b01001001, 0b01011111]),
            ("2041-03-04T00:00:00.123-10:15", vec![CP_DATETIME, 0b11110011, 0b00000001, 0b01010011, 0b00111001, 0b00000101, 0b11100010, 0b00110111, 0b01011101]),
            ("1970-01-01T00:00:00+00:00", vec![CP_DATETIME, 0b11110001, 0b10000001, 0b01101001, 0b11001110, 0b10100111, 0b11111110]),
            ("2017-05-03T05:52:03+00:00", vec![CP_DATETIME, 0b11101101, 0b10101000, 0b11100111, 0b11110010]),
            ("2017-05-03T15:52:03.923Z", vec![CP_DATETIME, 0b11110001, 0b10010110, 0b00010011, 0b00110100, 0b10111110, 0b10110100]),
            ("2017-05-03T15:52:31.123+10:00", vec![CP_DATETIME, 0b11110010, 0b10001011, 0b00001101, 0b11100100, 0b00101100, 0b11011001, 0b01011111]),
            ("2017-05-03T15:52:03Z", vec![CP_DATETIME, 0b11101101, 0b10100110, 0b10110101, 0b01110010]),
            ("2017-05-03T15:52:03-01:30", vec![CP_DATETIME, 0b11110001, 0b10000010, 0b11010011, 0b00110000, 0b10001000, 0b00010101]),
            ("2017-05-03T15:52:03.923+00:00", vec![CP_DATETIME, 0b11110001, 0b10010110, 0b00010011, 0b00110100, 0b10111110, 0b10110100]),
        ];
        for (dt_str, expected) in test_cases {
            // println!("--------------------\nTesting: {}", dt_str);
            let dt = DateTime::parse_from_rfc3339(dt_str).unwrap();
            let cpdt = CPDateTime(dt);
            {
                let serialized = to_vec(&cpdt).expect("serialization failed");
                // println!("Serialized: {serialized:x?}");
                assert_eq!(expected, serialized);
                let deserialized: CPDateTime = from_slice(&serialized).expect("deserialization failed");
                assert_eq!(cpdt, deserialized);
            }
            {
                let event = Event { timestamp: cpdt };
                let serialized = to_vec(&event).expect("serialization failed");
                // println!("Serialized: {serialized:x?}");
                let deserialized: Event = from_slice(&serialized).expect("deserialization failed");
                assert_eq!(event, deserialized);
            }
        }
    }
}
