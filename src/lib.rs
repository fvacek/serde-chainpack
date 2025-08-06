pub mod de;
pub mod error;
pub mod ser;
pub mod types;
pub mod cpdatetime;
pub mod cpdecimal;
mod rawbytes;
pub mod cpistruct;

// pub use cpdecimal::CPDecimal as Decimal;
// pub use cpdatetime::CPDateTime as DateTime;

pub fn hex_dump(data: &[u8]) -> String {
    let mut output = String::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        // Offset
        let offset = i * 16;
        output.push_str(&format!("{:08x}  ", offset));

        // Hex part
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02x} ", byte));
            if j == 7 {
                output.push(' '); // Extra space between 8-byte halves
            }
        }

        // Pad if less than 16 bytes
        if chunk.len() < 16 {
            let pad = 16 - chunk.len();
            for _ in 0..pad {
                output.push_str("   ");
            }
            if chunk.len() <= 8 {
                output.push(' ');
            }
        }

        // ASCII part
        output.push_str(" |");
        for byte in chunk {
            let c = *byte as char;
            if c.is_ascii_graphic() || c == ' ' {
                output.push(c);
            } else {
                output.push('.');
            }
        }
        output.push_str("|\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde::{Deserialize, Serialize};
    use crate::{cpdatetime::CPDateTime, cpdecimal::CPDecimal, cpistruct::CPIStruct, de::from_slice, ser::tests::to_vec};

    #[test]
    fn test_struct_serde() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestIStruct {
            #[serde(rename = "1")]
            age: i32,
            #[serde(rename = "2")]
            name: String,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct SubStruct {
            number: i32,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct MyStruct {
            numbers: Vec<i32>,
            both: (CPDateTime, i64),
            sub: SubStruct,
            timestamp: CPDateTime,
            decimal: CPDecimal,
            maybe_decimal: Option<CPDecimal>,
            istruct: CPIStruct<TestIStruct>,
            name: String,
            age: u32,
            salary: i32,
            weight: f64,
        }
        let dt = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap();
        let s = MyStruct {
            numbers: vec![1, 2, 3, 4, 5],
            both: (dt.into(), 42),
            sub: SubStruct { number: 42 },
            timestamp: dt.into(),
            decimal: CPDecimal::new(12345, 6),
            maybe_decimal: None,
            name: String::from("John Doe"),
            age: 30,
            salary: 50000,
            weight: 70.5,
            istruct: CPIStruct(TestIStruct{ age: 42, name: "Foo Bar".into() }),
        };
        let serialized = to_vec(&s).expect("serialization failed");
        // println!("Serialized: \n{}", hex_dump(&serialized));
        let deserialized: MyStruct = from_slice(&serialized).expect("deserialization failed");
        assert_eq!(s, deserialized);
    }
}
