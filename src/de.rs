use std::io::{Read, BufReader, BufRead};

use serde::de::{self, Visitor, SeqAccess, MapAccess};
use crate::error::{Result, Error};
use byteorder::{ReadBytesExt, BigEndian};

pub struct Deserializer<R> {
    reader: BufReader<R>,
    peeked: Option<u8>,
}

impl<R: Read> Deserializer<R> {
    pub fn from_reader(reader: R) -> Self {
        Deserializer {
            reader: BufReader::new(reader),
            peeked: None,
        }
    }

    fn peek_u8(&mut self) -> Result<u8> {
        if let Some(b) = self.peeked {
            return Ok(b);
        }
        let b = self.reader.read_u8()?;
        self.peeked = Some(b);
        Ok(b)
    }

    fn next_u8(&mut self) -> Result<u8> {
        if let Some(b) = self.peeked.take() {
            return Ok(b);
        }
        self.reader.read_u8()
            .map_err(Error::from)
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let type_byte = self.next_u8()?;
        match type_byte {
            0x00..=0x3F => visitor.visit_u64(type_byte as u64),
            0xC8 => visitor.visit_i64(self.reader.read_i64::<BigEndian>()?),
            0xC9 => visitor.visit_u64(self.reader.read_u64::<BigEndian>()?),
            0xCB => visitor.visit_f64(self.reader.read_f64::<BigEndian>()?),
            0xE0 => {
                let len = self.reader.read_u64::<BigEndian>()?;
                let mut buf = vec![0; len as usize];
                self.reader.read_exact(&mut buf)?;
                visitor.visit_byte_buf(buf)
            }
            0xE1 => {
                let mut buf = Vec::new();
                self.reader.read_until(0, &mut buf)?;
                buf.pop();
                visitor.visit_string(String::from_utf8(buf)?)
            }
            0xE2 => visitor.visit_seq(self),
            0xE3 => visitor.visit_map(self),
            0xFD => visitor.visit_bool(false),
            0xFE => visitor.visit_bool(true),
            0xFF => visitor.visit_unit(),
            _ => Err(Error::InvalidType),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_u8()? == 0xFF {
            self.next_u8()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, R: Read> SeqAccess<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.peek_u8()? == 0xFF {
            self.next_u8()?;
            return Ok(None);
        }
        seed.deserialize(&mut **self).map(Some)
    }
}

impl<'de, 'a, R: Read> MapAccess<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.peek_u8()? == 0xFF {
            self.next_u8()?;
            return Ok(None);
        }
        seed.deserialize(&mut **self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut **self)
    }
}
