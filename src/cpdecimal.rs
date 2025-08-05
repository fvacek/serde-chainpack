use serde::de::{IntoDeserializer, SeqAccess};
use serde::{Deserialize, Serialize};
use serde::{de, Deserializer, Serializer};
use std::fmt;

use crate::error::Error;
use crate::rawbytes::CP_RAWBYTES_NEWTYPE_STRUCT;
use crate::ser::serialize_raw_i64;
use crate::types::CP_DECIMAL;

#[derive(Debug, PartialEq)]
pub struct CPDecimal {
    mantissa: i64,
    exponent: i8,
}
pub(crate) const CP_DECIMAL_NEWTYPE_STRUCT: &str = "CPDecimal";

impl CPDecimal {
    pub fn new(mantissa: i64, exponent: i8) -> Self {
        Self { mantissa, exponent }
    }
    pub fn mantissa(&self) -> i64 {
        self.mantissa
    }
    pub fn exponent(&self) -> i8 {
        self.exponent
    }
    pub fn to_f64(&self) -> f64 {
        self.mantissa as f64 * 10f64.powi(self.exponent as i32)
    }
}

impl Serialize for CPDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = vec![];
        bytes.push(CP_DECIMAL);
        serialize_raw_i64(&mut bytes, self.mantissa).map_err(serde::ser::Error::custom)?;
        serialize_raw_i64(&mut bytes, self.exponent as i64).map_err(serde::ser::Error::custom)?;
        serializer.serialize_newtype_struct(CP_RAWBYTES_NEWTYPE_STRUCT, &crate::rawbytes::RawBytes(&bytes))
    }
}

pub(crate) struct DecimalDeserializer {
    pub(crate) mantissa: i64,
    pub(crate) exponent: i64,
    pub(crate) state: u8,
}

impl<'de, 'a> SeqAccess<'de> for &'a mut DecimalDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> crate::error::Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.state {
            0 => {
                self.state += 1;
                seed.deserialize(self.mantissa.into_deserializer()).map(Some)
            }
            1 => {
                self.state += 1;
                seed.deserialize(self.exponent.into_deserializer()).map(Some)
            }
            _ => Ok(None),
        }
    }
}

impl<'de> Deserialize<'de> for CPDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(CP_DECIMAL_NEWTYPE_STRUCT, DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> de::Visitor<'de> for DecimalVisitor {
    type Value = CPDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ChainPack Decimal")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>
    {
        let mantissa = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let exponent = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok(CPDecimal { mantissa, exponent })
    }
}

#[cfg(test)]
mod tests {
    use crate::{de::from_slice, types::CP_DECIMAL};
    use super::CPDecimal;

    #[test]
    fn test_decimal_serde() {
        let test_cases = vec![
            (CPDecimal::new(1,2), vec![CP_DECIMAL, 1, 2]),
            (CPDecimal::new(1,-2), vec![CP_DECIMAL, 1, 0b0100_0010]),
        ];
        for (dec, expected) in test_cases {
            // println!("--------------------\nTesting: {dec:?}");
            let serialized = crate::ser::tests::to_vec(&dec).expect("serialization failed");
            // println!("Serialized: {serialized:x?}");
            assert_eq!(expected, serialized);
            let deserialized: CPDecimal = from_slice(&serialized).expect("deserialization failed");
            assert_eq!(dec, deserialized);
        }
    }
}
