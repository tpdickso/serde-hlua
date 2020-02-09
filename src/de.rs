
//! Deserialization from lua values to rust values.

use std::error;
use std::fmt;
use std::iter::ExactSizeIterator;
use std::vec::IntoIter;

#[cfg(feature = "base64-bytes")]
use base64;
use hlua::AnyLuaValue;
use serde;
use serde::de::{Deserializer, Visitor};

/// A deserializer over an `AnyLuaValue` that can deserialize it to a provided
/// format.
#[derive(Debug, Clone)]
pub struct LuaDeserializer(AnyLuaValue);

impl LuaDeserializer {
    /// Return a deserializer that can deserialize a value from the provided
    /// lua data.
    pub fn new(value: AnyLuaValue) -> LuaDeserializer {
        LuaDeserializer(value)
    }
}

impl<'de> Deserializer<'de> for LuaDeserializer {
    type Error = LuaDeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaString(string) => visitor.visit_string(string),
            AnyLuaValue::LuaAnyString(_) => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other("non-utf-8 string"),
                &visitor
            )),
            AnyLuaValue::LuaNumber(number) => visitor.visit_f64(number),
            AnyLuaValue::LuaBoolean(boolean) => visitor.visit_bool(boolean),
            AnyLuaValue::LuaArray(array) => match is_vec(array) {
                Ok(array) => visitor.visit_seq(LuaSeqAccess(array.into_iter())),
                Err(map) => visitor.visit_map(LuaMapAccess(map.into_iter(), None))
            },
            AnyLuaValue::LuaNil => visitor.visit_unit(),
            _=> Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaBoolean(boolean) => visitor.visit_bool(boolean),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as i8 as f64 == number
            ) => visitor.visit_i8(number as i8),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as i16 as f64 == number
            ) => visitor.visit_i16(number as i16),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as i32 as f64 == number
            ) => visitor.visit_i32(number as i32),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as i64 as f64 == number
            ) => visitor.visit_i64(number as i64),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as u8 as f64 == number
            ) => visitor.visit_u8(number as u8),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as u16 as f64 == number
            ) => visitor.visit_u16(number as u16),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as u32 as f64 == number
            ) => visitor.visit_u32(number as u32),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) if (
                number as u64 as f64 == number
            ) => visitor.visit_u64(number as u64),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) => visitor.visit_f32(number as f32),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNumber(number) => visitor.visit_f64(number),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaString(ref string) => {
                let mut char_iterator = string.chars();
                if let Some(character) = char_iterator.next() {
                    if char_iterator.next().is_some() {
                        Err(serde::de::Error::invalid_length(
                            2 + char_iterator.count(),
                            &visitor
                        ))
                    } else {
                        visitor.visit_char(character)
                    }
                } else {
                    Err(serde::de::Error::invalid_length(0, &visitor))
                }
            }
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaString(ref string) => visitor.visit_str(string.as_ref()),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaString(string) => visitor.visit_string(string),
            _ => Err(error(&self.0, &visitor))
        }
    }

    #[cfg(not(feature = "base64-bytes"))]
    fn deserialize_bytes<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        Err(serde::de::Error::custom(
            "cannot deserialize bytes; compile with 'base64-bytes'"
        ))
    }

    #[cfg(feature = "base64-bytes")]
    fn deserialize_bytes<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaString(ref string) => {
                match base64::decode(string) {
                    Ok(bytes) => visitor.visit_bytes(bytes.as_ref()),
                    Err(_) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("non-base64 data"),
                        &visitor
                    ))
                }
            },
            _ => Err(error(&self.0, &visitor))
        }
    }

    #[cfg(not(feature = "base64-bytes"))]
    fn deserialize_byte_buf<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        Err(serde::de::Error::custom(
            "cannot deserialize byte_buf; compile with 'base64-bytes'"
        ))
    }

    #[cfg(feature = "base64-bytes")]
    fn deserialize_byte_buf<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaString(ref string) => {
                match base64::decode(string) {
                    Ok(bytes) => visitor.visit_byte_buf(bytes),
                    Err(_) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("non-base64 data"),
                        &visitor
                    ))
                }
            },
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaNil => visitor.visit_none(),
            _ => visitor.visit_some(LuaDeserializer(self.0))
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNil => visitor.visit_unit(),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match &self.0 {
            &AnyLuaValue::LuaNil => visitor.visit_unit(),
            _ => Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaArray(array) => {
                match is_vec(array) {
                    Ok(array) => visitor.visit_seq(LuaSeqAccess(array.into_iter())),
                    Err(_) => Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Map,
                        &visitor
                    ))
                }
            },
            _=> Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaArray(array) => {
                if array.len() != len {
                    return Err(serde::de::Error::invalid_length(array.len(), &visitor));
                }
                match is_vec(array) {
                    Ok(array) => visitor.visit_seq(LuaSeqAccess(array.into_iter())),
                    Err(_) => Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Map,
                        &visitor
                    ))
                }
            },
            _=> Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaArray(array) => {
                visitor.visit_map(LuaMapAccess(array.into_iter(), None))
            },
            _=> Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        match self.0 {
            AnyLuaValue::LuaString(identifier) => {
                visitor.visit_enum(LuaEnumAccess(
                    AnyLuaValue::LuaString(identifier),
                    AnyLuaValue::LuaNil
                ))
            },
            AnyLuaValue::LuaArray(array) => {
                if array.len() != 1 {
                    return Err(serde::de::Error::invalid_length(array.len(), &visitor));
                }
                let (key, value) = array.into_iter().next().unwrap();
                visitor.visit_enum(LuaEnumAccess(key, value))
            },
            _=> Err(error(&self.0, &visitor))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_any(visitor)
    }
}

/// Return `Ok(sorted)` if the input array is an actual array (keys from
/// 1..N) and `Err(original array)` otherwise.
fn is_vec(array: Vec<(AnyLuaValue, AnyLuaValue)>) -> Result<
    Vec<(AnyLuaValue, AnyLuaValue)>,
    Vec<(AnyLuaValue, AnyLuaValue)>
> {
    if array.iter().any(|&(ref key, _)| match key {
        &AnyLuaValue::LuaNumber(_) => false,
        _ => true
    }) {
        return Err(array);
    }

    let mut sorted = array.clone();
    sorted.sort_by_key(|&(ref index, _)| match index {
        &AnyLuaValue::LuaNumber(number) => number as usize,
        _ => unreachable!()
    });

    let mut is_array = true;
    for (index, &(ref key, _)) in array.iter().enumerate() {
        if !match key {
            &AnyLuaValue::LuaNumber(number) if (
                number as usize as f64 == number &&
                number == (index + 1) as f64
            ) => true,
            _ => false
        } {
            is_array = false;
            break;
        }
    }

    if is_array {
        Ok(sorted)
    } else {
        Err(array)
    }
}

/// Sequential access over a `LuaArray`.
// The vector used to create this must be a table with keys from 1 to N, and
// must be sorted by key. The iterator given is the remaining key-values in
// the array to be yielded, where the keys are ignored.
pub struct LuaSeqAccess(IntoIter<(AnyLuaValue, AnyLuaValue)>);

impl<'de> serde::de::SeqAccess<'de> for LuaSeqAccess {
    type Error = LuaDeserializeError;

    fn next_element_seed<T>(
        &mut self,
        seed: T
    ) -> DeResult<Option<T::Value>>
        where T: serde::de::DeserializeSeed<'de>
    {
        Ok(match self.0.next() {
            Some((_, value)) => Some(seed.deserialize(LuaDeserializer(value))?),
            None => None
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(ExactSizeIterator::len(&self.0))
    }
}

/// Map access over a `LuaArray`.
// The first element is the remaining key-value pairs of the map to yield,
// and the second element is the value in the case where a key has been
// yielded but not its value.
pub struct LuaMapAccess(IntoIter<(AnyLuaValue, AnyLuaValue)>, Option<AnyLuaValue>);

impl<'de> serde::de::MapAccess<'de> for LuaMapAccess {
    type Error = LuaDeserializeError;

    fn next_key_seed<K>(
        &mut self,
        seed: K
    ) -> DeResult<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
    {
        Ok(match self.0.next() {
            Some((key, value)) => {
                self.1 = Some(value);
                Some(seed.deserialize(LuaDeserializer(key))?)
            },
            None => None
        })
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V
    ) -> DeResult<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        seed.deserialize(LuaDeserializer(self.1.take().unwrap()))
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> DeResult<Option<(K::Value, V::Value)>>
        where K: serde::de::DeserializeSeed<'de>,
              V: serde::de::DeserializeSeed<'de>
    {
        Ok(match self.0.next() {
            Some((key, value)) => {
                Some((
                    kseed.deserialize(LuaDeserializer(key))?,
                    vseed.deserialize(LuaDeserializer(value))?
                ))
            },
            None => None
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(ExactSizeIterator::len(&self.0))
    }
}

/// Variant access over a `LuaArray` of one item.
pub struct LuaEnumAccess(AnyLuaValue, AnyLuaValue);

impl<'de> serde::de::EnumAccess<'de> for LuaEnumAccess {
    type Error = LuaDeserializeError;
    type Variant = LuaVariantAccess;

    fn variant_seed<V>(
        self,
        seed: V
    ) -> DeResult<(V::Value, Self::Variant)>
        where V: serde::de::DeserializeSeed<'de>
    {
        Ok((seed.deserialize(LuaDeserializer(self.0))?, LuaVariantAccess(self.1)))
    }
}

/// Variant access over a `LuaArray` of one item.
pub struct LuaVariantAccess(AnyLuaValue);

impl<'de> serde::de::VariantAccess<'de> for LuaVariantAccess {
    type Error = LuaDeserializeError;

    fn unit_variant(self) -> DeResult<()> {
        match &self.0 {
            &AnyLuaValue::LuaNil => Ok(()),
            _ => Err(error(&self.0, &"unit variant"))
        }
    }

    fn newtype_variant_seed<T>(
        self,
        seed: T
    ) -> DeResult<T::Value>
        where T: serde::de::DeserializeSeed<'de>
    {
        seed.deserialize(LuaDeserializer(self.0))
    }

    fn tuple_variant<V>(
        self,
        len: usize,
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        LuaDeserializer(self.0).deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V
    ) -> DeResult<V::Value>
        where V: Visitor<'de>
    {
        LuaDeserializer(self.0).deserialize_struct("", fields, visitor)
    }
}

fn error<E>(value: &AnyLuaValue, expected: &E) -> LuaDeserializeError
    where E: serde::de::Expected
{
    serde::de::Error::invalid_type(
        match value {
            &AnyLuaValue::LuaString(ref string) => serde::de::Unexpected::Str(
                string.as_ref()
            ),
            &AnyLuaValue::LuaAnyString(ref bytes) => serde::de::Unexpected::Bytes(
                bytes.0.as_ref()
            ),
            &AnyLuaValue::LuaNumber(number) => serde::de::Unexpected::Float(number),
            &AnyLuaValue::LuaBoolean(boolean) => serde::de::Unexpected::Bool(boolean),
            &AnyLuaValue::LuaArray(_) => serde::de::Unexpected::Map,
            &AnyLuaValue::LuaNil => serde::de::Unexpected::Unit,
            &AnyLuaValue::LuaOther => serde::de::Unexpected::Other("unserializable")
        },
        expected
    )
}

/// A result returned by lua deserialization.
pub type DeResult<T> = Result<T, LuaDeserializeError>;

/// An error returned by lua deserialization.
#[derive(Debug, Clone)]
pub struct LuaDeserializeError(String);

impl fmt::Display for LuaDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl error::Error for LuaDeserializeError {
}

impl serde::de::Error for LuaDeserializeError {
    fn custom<T>(msg: T) -> Self
        where T: fmt::Display
    {
        LuaDeserializeError(format!("{}", msg))
    }
}

#[cfg(test)]
mod tests {
    use std;
    use hlua;

    use std::collections::{BTreeMap, BTreeSet};

    use ::from_lua;

    fn procure(value: &str) -> hlua::AnyLuaValue {
        let mut lua = hlua::Lua::new();
        lua.execute::<hlua::AnyLuaValue>(&format!("return {}", value)).unwrap()
    }

    #[test]
    fn boolean() {
        assert_eq!(true, from_lua(procure("true")).unwrap());
        assert_eq!(false, from_lua(procure("false")).unwrap());
        assert!(from_lua::<bool>(procure("1.0")).is_err());
        assert!(from_lua::<bool>(procure("{}")).is_err());
    }

    #[test]
    fn number() {
        assert_eq!(1.0f32, from_lua(procure("1.0")).unwrap());
        assert_eq!(19, from_lua(procure("19.0")).unwrap());
        assert_eq!(-45i8, from_lua(procure("-45")).unwrap());
        assert_eq!(std::f32::INFINITY, from_lua(procure("1/0")).unwrap());
        assert_eq!(std::f32::NEG_INFINITY, from_lua(procure("-1/0")).unwrap());
        assert!(from_lua::<f32>(procure("0/0")).unwrap().is_nan());
        assert!(from_lua::<u32>(procure("1.5")).is_err());
        assert!(from_lua::<u32>(procure("1/0")).is_err());
        assert!(from_lua::<u32>(procure("0/0")).is_err());
        assert!(from_lua::<u32>(procure("{}")).is_err());
        assert!(from_lua::<f32>(procure("false")).is_err());
    }

    #[test]
    fn string() {
        assert_eq!("good morning", from_lua::<String>(procure("'good morning'")).unwrap());
        assert_eq!("κόσμε", from_lua::<String>(procure("'κόσμε'")).unwrap());
        assert_eq!("你好，世界", from_lua::<String>(procure("'你好，世界'")).unwrap());
        assert!(from_lua::<String>(procure("0/0")).is_err());
        assert!(from_lua::<String>(procure("12")).is_err());
        assert!(from_lua::<String>(procure("false")).is_err());
    }

    #[test]
    fn sequence() {
        use std::iter::FromIterator;

        assert_eq!(
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
            from_lua::<Vec<String>>(procure("{'a', 'b', 'c'}")).unwrap()
        );

        assert_eq!(
            BTreeSet::from_iter(vec![
                "a".to_owned(),
                "b".to_owned(),
                "c".to_owned()
            ].into_iter()),
            from_lua::<BTreeSet<String>>(procure("{'a', 'b', 'c'}")).unwrap()
        );
    }

    #[test]
    fn mapping() {
        use std::iter::FromIterator;

        assert_eq!(
            BTreeMap::from_iter(vec![
                ("first".to_owned(), 1.0),
                ("second".to_owned(), 2.0),
                ("third".to_owned(), 3.25)
            ].into_iter()),
            from_lua::<BTreeMap<String, f32>>(procure(
                "{ first = 1, second = 2, third = 3.25}"
            )).unwrap()
        );

        assert!(
            from_lua::<BTreeMap<String, f32>>(procure(
                "{ first = 1, second = 2, third = '3.25'}"
            )).is_err()!
        );
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename_all = "snake_case")]
    enum UnitEnum {
        First,
        Second,
        AndTheThird
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename_all = "snake_case")]
    enum ComplexEnum {
        Scalar,
        Tuple(f32, f32),
        Struct {
            #[serde(default)]
            name: String,
            contents: SimpleStruct
        }
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct SimpleStruct {
        scalar: f32,
        string: String,
        vector: Vec<u32>
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct NestedStruct {
        title: String,
        first_name: String,
        last_name: String,
        data: ComplexEnum
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct FailUnitStruct {
        value: ()
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct SuccessUnitStruct {
        #[serde(skip)]
        value: ()
    }

    #[test]
    fn structs() {
        assert_eq!(
            SimpleStruct { scalar: 1.0, string: "Hi!".to_owned(), vector: vec![1, 2, 9] },
            from_lua::<SimpleStruct>(procure(
                "{ scalar = 1, string = 'Hi!', vector = { 1, 2, 9 } }"
            )).unwrap()
        );

        assert!(
            from_lua::<SimpleStruct>(procure(
                "{ scalar = 1, string = 'Hi!', vector = { 1.1, 2, 9 } }"
            )).is_err()
        );

        assert_eq!(
            NestedStruct {
                title: "Dr.".to_owned(),
                first_name: "Loretta".to_owned(),
                last_name: "Spanx".to_owned(),
                data: ComplexEnum::Struct {
                    name: "Beastly!".to_owned(),
                    contents: SimpleStruct {
                        scalar: 1.0,
                        string: "Hi!".to_owned(),
                        vector: vec![1, 2, 9]
                    },
                }
            },
            from_lua::<NestedStruct>(procure(
                "{ title = 'Dr.',
                   first_name = 'Loretta',
                   last_name = 'Spanx',
                   data = { struct = { name = 'Beastly!',
                                       contents = { scalar = 1,
                                                    string = 'Hi!',
                                                    vector = { 1, 2, 9 } } } } }"
            )).unwrap()
        );

        assert!(from_lua::<NestedStruct>(procure(
            "{ title = 'Dr.',
               first_name = 'Loretta',
               last_name = 'Spanx',
               data = { struct = { name = 'Beastly!',
                                   contents = { scalar = 1,
                                                string = 5,
                                                vector = { 1, 2, 9 } } } } }"
        )).is_err());
    }

    #[test]
    fn enums() {
        assert_eq!(UnitEnum::First, from_lua::<UnitEnum>(procure("'first'")).unwrap());
        assert_eq!(UnitEnum::Second, from_lua::<UnitEnum>(procure("'second'")).unwrap());

        assert_eq!(
            UnitEnum::AndTheThird,
            from_lua::<UnitEnum>(procure("'and_the_third'")).unwrap()
        );

        assert!(from_lua::<UnitEnum>(procure("'fourth'")).is_err());

        assert_eq!(ComplexEnum::Scalar, from_lua::<ComplexEnum>(procure("'scalar'")).unwrap());

        assert_eq!(
            ComplexEnum::Tuple(1.3, 3.1),
            from_lua::<ComplexEnum>(procure("{ tuple = { 1.3, 3.1 } }")).unwrap()
        );

        assert_eq!(
            ComplexEnum::Struct {
                name: "Beastly!".to_owned(),
                contents: SimpleStruct {
                    scalar: 1.0,
                    string: "Hi!".to_owned(),
                    vector: vec![1, 2, 9]
                }
            },
            from_lua::<ComplexEnum>(procure(
                "{ struct = { name = 'Beastly!',
                              contents = { scalar = 1,
                                           string = 'Hi!',
                                           vector = { 1, 2, 9 } } } }"
            )).unwrap()
        );

        assert_eq!(
            vec![
                ComplexEnum::Struct {
                    name: "Maria".to_owned(),
                    contents: SimpleStruct {
                        scalar: 1.1,
                        string: "arglebargle".to_owned(),
                        vector: vec![1]
                    }
                },
                ComplexEnum::Struct {
                    name: "Chelsea".to_owned(),
                    contents: SimpleStruct {
                        scalar: 1.11,
                        string: "French".to_owned(),
                        vector: vec![99, 99]
                    }
                },
                ComplexEnum::Tuple(4.0, 3.0),
                ComplexEnum::Scalar,
                ComplexEnum::Struct {
                    name: "Baljeet".to_owned(),
                    contents: SimpleStruct {
                        scalar: 1.11,
                        string: "corn on the 好 cob".to_owned(),
                        vector: vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
                    }
                }
            ],
            from_lua::<Vec<ComplexEnum>>(procure(
                "{
                    { struct = { name = 'Maria',
                                 contents = { scalar = 1.1,
                                              string = 'arglebargle',
                                              vector = { 1 } } } },
                    { struct = { name = 'Chelsea',
                                 contents = { scalar = 1.11,
                                              string = 'French',
                                              vector = { 99, 99 } } } },
                    { tuple = { 4, 3.0 } },
                    'scalar',
                    { struct = { name = 'Baljeet',
                                 contents = { scalar = 1.11,
                                              string = 'corn on the 好 cob',
                                              vector = { 10, 9, 8, 7, 6, 5, 4, 3, 2, 1 } } } }
                }"
            )).unwrap()
        );
    }

    #[test]
    fn unit_limitations() {
        assert!(from_lua::<FailUnitStruct>(procure("{}")).is_err());
        assert!(from_lua::<SuccessUnitStruct>(procure("{}")).is_ok());
        assert!(from_lua::<FailUnitStruct>(procure("{ value = nil }")).is_err());
        assert!(from_lua::<SuccessUnitStruct>(procure("{ value = nil }")).is_ok());
    }
}

