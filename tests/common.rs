use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_chainpack::{de::Deserializer, ser::Serializer, types::{CP_BLOB, CP_DOUBLE, CP_INT, CP_LIST, CP_MAP, CP_NULL, CP_STRING, CP_TERM, CP_UINT}};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    a: i32,
    b: String,
}

#[test]
fn test_bool() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_bool(&mut serializer, true).unwrap();
    assert_eq!(buffer, vec![0xFE]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = bool::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, true);
}

#[test]
fn test_i64() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_i64(&mut serializer, 1234567890).unwrap();
    assert_eq!(
        buffer,
        vec![CP_INT, 0xF0, 0x49, 0x96, 0x02, 0xD2]
    );

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = i64::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, 1234567890);
}

#[test]
fn test_u64() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_u64(&mut serializer, 1234567890).unwrap();
    assert_eq!(buffer, vec![CP_UINT, 0xF0, 0x49, 0x96, 0x02, 0xD2]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = u64::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, 1234567890);
}

#[test]
fn test_str() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_str(&mut serializer, "hello").unwrap();
    assert_eq!(buffer, vec![CP_STRING, 5, b'h', b'e', b'l', b'l', b'o']);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = String::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, "hello");
}

#[test]
fn test_f32() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_f32(&mut serializer, 123.456_f32).unwrap();
    assert_eq!(buffer, vec![CP_DOUBLE, 0x79, 0xE9, 0xF6, 0x42]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = f32::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, 123.456_f32);
}

#[test]
fn test_bytes() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_bytes(&mut serializer, &[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(buffer, vec![CP_BLOB, 5, 1, 2, 3, 4, 5]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = ByteBuf::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_option() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_some(&mut serializer, &123).unwrap();
    assert_eq!(buffer, vec![CP_INT, 0x80, 0x7b]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: Option<i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, Some(123));

    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_none(&mut serializer).unwrap();
    assert_eq!(buffer, vec![CP_NULL]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: Option<i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, None);
}

#[test]
fn test_unit() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_unit(&mut serializer).unwrap();
    assert_eq!(buffer, vec![CP_NULL]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: () = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, ());
}

#[test]
fn test_seq() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    let mut seq = serde::Serializer::serialize_seq(&mut serializer, Some(3)).unwrap();
    serde::ser::SerializeSeq::serialize_element(&mut seq, &1).unwrap();
    serde::ser::SerializeSeq::serialize_element(&mut seq, &2).unwrap();
    serde::ser::SerializeSeq::serialize_element(&mut seq, &3).unwrap();
    serde::ser::SerializeSeq::end(seq).unwrap();
    assert_eq!(buffer, vec![CP_LIST, 0x41, 0x42, 0x43, CP_TERM]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: Vec<i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, vec![1, 2, 3]);
}

#[test]
fn test_map() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    let mut map = serde::Serializer::serialize_map(&mut serializer, Some(2)).unwrap();
    serde::ser::SerializeMap::serialize_entry(&mut map, "a", &1).unwrap();
    serde::ser::SerializeMap::serialize_entry(&mut map, "b", &2).unwrap();
    serde::ser::SerializeMap::end(map).unwrap();
    assert_eq!(buffer, vec![CP_MAP, CP_STRING, 1, b'a', 0x41, CP_STRING, 1, b'b', 0x42, CP_TERM]);
    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: std::collections::HashMap<String, i32> =
        serde::Deserialize::deserialize(&mut deserializer).unwrap();
    let mut expected = std::collections::HashMap::new();
    expected.insert("a".to_string(), 1);
    expected.insert("b".to_string(), 2);
    assert_eq!(value, expected);
}

#[test]
fn test_struct() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    let test_struct = TestStruct {
        a: 1,
        b: "hello".to_string(),
    };
    test_struct.serialize(&mut serializer).unwrap();
    assert_eq!(
        buffer,
        vec![
            CP_MAP, CP_STRING, 1, b'a', 0x41, CP_STRING, 1, b'b', CP_STRING, 5,b'h', b'e', b'l', b'l', b'o', CP_TERM
        ]
    );

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: TestStruct = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, test_struct);
}

#[test]
fn test_uint_examples() {
    let test_cases = vec![
        (2u64, vec![0x02]),
        (0x10u64, vec![0x10]),
        (127u64, vec![CP_UINT, 0b01111111]),
        (0x80u64, vec![CP_UINT, 0b10000000, 0b10000000]),
        (0x200u64, vec![CP_UINT, 0b10000010, 0b00000000]),
        (0x1000u64, vec![CP_UINT, 0b10010000, 0b00000000]),
        (0x8000u64, vec![CP_UINT, 0xC0, 0x80, 0x00]),
        (0x100000u64, vec![CP_UINT, 0b11010000, 0x00, 0x00]),
        (0x800000u64, vec![CP_UINT, 0xE0, 0x80, 0x00, 0x00]),
        (0x2000000u64, vec![CP_UINT, 0xE2, 0x00, 0x00, 0x00]),
        (0x10000000u64, vec![CP_UINT, 0b11110000, 0b00010000, 0x00, 0x00, 0x00]),
        (0x10_0000_0000u64, vec![CP_UINT, 0b11110001, 0b00010000, 0x00, 0x00, 0x00, 0x00]),
        (0x1000_0000_0000u64, vec![ CP_UINT, 0b11110010, 0b00010000, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (0x8000_0000_0000u64, vec![ CP_UINT, 0b11110010, 0b10000000, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (0x10_0000_0000_0000u64, vec![ CP_UINT, 0b11110011, 0b00010000, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ];

    for (value, expected) in test_cases {
        println!("value: 0x{value:x}, expected: {expected:x?}");
        let mut buffer = Vec::new();
        let mut serializer = Serializer::new(&mut buffer);
        serde::Serializer::serialize_u64(&mut serializer, value).unwrap();
        assert_eq!(buffer, expected);

        let mut deserializer = Deserializer::from_reader(&buffer[..]);
        let deserialized_value = u64::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized_value, value);
    }
}

#[test]
fn test_int_examples() {
    let test_cases = vec![
        (-64_i64, vec![CP_INT, 0b10100000, 0b01000000]),
        (4, vec![0b01000100]),
        (16_i64, vec![0b01010000]),
        (64_i64, vec![CP_INT, 0b10000000, 0b01000000]),
        (1024_i64, vec![CP_INT, 0b10000100, 0b00000000]),
        (4096_i64, vec![CP_INT, 0b10010000, 0b00000000]),
        (16384_i64, vec![CP_INT, 0b11000000, 0b01000000, 0b00000000]),
        (262144_i64, vec![CP_INT, 0b11000100, 0b00000000, 0b00000000]),
        (1048576_i64, vec![CP_INT, 0b11100000, 0b00010000, 0b00000000, 0b00000000]),
        (4194304_i64, vec![CP_INT, 0b11100000, 0b01000000, 0b00000000, 0b00000000]),
        (67108864_i64, vec![CP_INT, 0b11100100, 0b00000000, 0b00000000, 0b00000000]),
        (268435456_i64, vec![CP_INT, 0b11110000, 0b00010000, 0b00000000, 0b00000000, 0b00000000]),
        (1073741824_i64, vec![CP_INT, 0b11110000, 0b01000000, 0b00000000, 0b00000000, 0b00000000]),
        (17179869184_i64, vec![CP_INT, 0b11110001, 0b00000100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (68719476736_i64, vec![CP_INT, 0b11110001, 0b00010000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (274877906944_i64, vec![CP_INT, 0b11110001, 0b01000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (4398046511104_i64, vec![CP_INT, 0b11110010, 0b00000100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (17592186044416_i64, vec![CP_INT, 0b11110010, 0b00010000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (70368744177664_i64, vec![CP_INT, 0b11110010, 0b01000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (-4_i64, vec![CP_INT, 0b01000100]),
        (-16_i64, vec![CP_INT, 0b01010000]),
        (-64_i64, vec![CP_INT, 0b10100000, 0b01000000]),
        (-1024_i64, vec![CP_INT, 0b10100100, 0b00000000]),
        (-4096_i64, vec![CP_INT, 0b10110000, 0b00000000]),
        (-16384_i64, vec![CP_INT, 0b11010000, 0b01000000, 0b00000000]),
        (-262144_i64, vec![CP_INT, 0b11010100, 0b00000000, 0b00000000]),
    ];

    for (value, expected) in test_cases {
        // println!("value: {value}, expected: {expected:x?}");
        let mut buffer = Vec::new();
        let mut serializer = Serializer::new(&mut buffer);
        serde::Serializer::serialize_i64(&mut serializer, value).unwrap();
        assert_eq!(expected, buffer);

        let mut deserializer = Deserializer::from_reader(&buffer[..]);
        let deserialized_value = i64::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized_value, value);
    }
}
