use std::fmt;
use serde::{de, de::{Deserialize, Deserializer, IntoDeserializer, MapAccess, Visitor}, Serialize};
use serde::de::value::MapAccessDeserializer;

#[derive(Debug, PartialEq, Serialize)]
pub struct CPIStruct<T>(pub T);

pub(crate) const CP_ISTRUCT_NEWTYPE_STRUCT: &str = "CPIStruct";

impl<'de, T> Deserialize<'de> for CPIStruct<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(CP_ISTRUCT_NEWTYPE_STRUCT, IStructVisitor(std::marker::PhantomData))
    }
}

struct IStructVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T: de::Deserialize<'de>> Visitor<'de> for IStructVisitor<T> {
    type Value = CPIStruct<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ChainPack CPIStruct")
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let de = MapAccessDeserializer::new(IntKeyMapAccess(map));
        T::deserialize(de).map(CPIStruct)
    }
}

struct IntKeyMapAccess<A>(A);

impl<'de, A> MapAccess<'de> for IntKeyMapAccess<A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.0.next_key::<i64>()? {
            Some(int_key) => {
                let key_str = int_key.to_string();
                seed.deserialize(key_str.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.0.next_value_seed(seed)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::{cpistruct::CPIStruct, de::Deserializer, ser::Serializer, types::{CP_IMAP, CP_STRING, CP_TERM}};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestIStruct {
        #[serde(rename = "1")]
        foo: i32,
        #[serde(rename = "3")]
        bar: String,
    }

    #[test]
    fn test_istruct() {
        let mut buffer = Vec::new();
        let mut serializer = Serializer::new(&mut buffer);
        let test_struct = CPIStruct(TestIStruct {
            foo: 1,
            bar: "hello".to_string(),
        });
        test_struct.serialize(&mut serializer).unwrap();
        assert_eq!(
            buffer,
            vec![
                CP_IMAP, 0x41, 0x41, 0x43, CP_STRING, 5,b'h', b'e', b'l', b'l', b'o', CP_TERM
            ]
        );

        let mut deserializer = Deserializer::from_reader(&buffer[..]);
        let value: CPIStruct<TestIStruct> = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(value, test_struct);
    }

}
