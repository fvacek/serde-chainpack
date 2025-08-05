pub mod de;
pub mod error;
pub mod ser;
pub mod types;
pub mod cpdatetime;
pub mod cpdecimal;
mod rawbytes;

// pub use cpdecimal::CPDecimal as Decimal;
// pub use cpdatetime::CPDateTime as DateTime;

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde::{Deserialize, Serialize};
    use crate::{cpdatetime::CPDateTime, cpdecimal::CPDecimal, de::from_slice, ser::tests::to_vec};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MyStruct {
        timestamp: CPDateTime,
        decimal: CPDecimal,
        maybe_decimal: Option<CPDecimal>,
        both: (CPDecimal, CPDateTime),
    }

    #[test]
    fn test_struct_serde() {
        let dt = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap();
        let s = MyStruct { timestamp: dt.into(), decimal: CPDecimal::new(12345, 6), maybe_decimal: None, both: (CPDecimal::new(0, -6), dt.into()) };
        let serialized = to_vec(&s).expect("serialization failed");
        // println!("Serialized: {serialized:x?}");
        let deserialized: MyStruct = from_slice(&serialized).expect("deserialization failed");
        assert_eq!(s, deserialized);
    }
}
