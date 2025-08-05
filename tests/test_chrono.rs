use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_chainpack::{cpdatetime::CPDateTime, de::from_slice, ser::to_vec, types::CP_DATETIME};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Event {
    timestamp: CPDateTime,
}

#[test]
fn test_datetime_serialization_round_trip() {
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
