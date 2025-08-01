use serde_chainpack::{de::Deserializer, ser::Serializer};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

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
    assert_eq!(buffer, vec![0xC8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x96, 0x02, 0xD2]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = i64::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, 1234567890);
}

#[test]
fn test_u64() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_u64(&mut serializer, 1234567890).unwrap();
    assert_eq!(buffer, vec![0xC9, 0x00, 0x00, 0x00, 0x00, 0x49, 0x96, 0x02, 0xD2]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = u64::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, 1234567890);
}



#[test]
fn test_str() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_str(&mut serializer, "hello").unwrap();
    assert_eq!(buffer, vec![0xE1, b'h', b'e', b'l', b'l', b'o', 0]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = String::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, "hello");
}

#[test]
fn test_bytes() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_bytes(&mut serializer, &[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(buffer, vec![0xE0, 0, 0, 0, 0, 0, 0, 0, 5, 1, 2, 3, 4, 5]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value = ByteBuf::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_option() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_some(&mut serializer, &123).unwrap();
    assert_eq!(buffer, vec![0xC8, 0, 0, 0, 0, 0, 0, 0, 123]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: Option<i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, Some(123));

    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_none(&mut serializer).unwrap();
    assert_eq!(buffer, vec![0xFF]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: Option<i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, None);
}

#[test]
fn test_unit() {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer);
    serde::Serializer::serialize_unit(&mut serializer).unwrap();
    assert_eq!(buffer, vec![0xFF]);

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
    assert_eq!(buffer, vec![0xE2, 1, 2, 3, 0xFF]);

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
    assert_eq!(buffer, vec![0xE3, 0xE1, b'a', 0, 1, 0xE1, b'b', 0, 2, 0xFF]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: std::collections::HashMap<String, i32> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
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
    assert_eq!(buffer, vec![0xE3, 0xE1, b'a', 0, 1, 0xE1, b'b', 0, 0xE1, b'h', b'e', b'l', b'l', b'o', 0, 0xFF]);

    let mut deserializer = Deserializer::from_reader(&buffer[..]);
    let value: TestStruct = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    assert_eq!(value, test_struct);
}
