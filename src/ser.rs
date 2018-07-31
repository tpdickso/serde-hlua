
//! Serialization from rust values to lua values.

use std::error;
use std::fmt;
use std::marker::PhantomData;

#[cfg(feature = "base64-bytes")]
use base64;
use hlua::AnyLuaValue;
use serde;
use serde::Serialize;
use serde::ser::Serializer;

// The phantom data here is just to make the type unconstructable outside of
// this crate, as we want to be able to potentially add fields in the future
// without it being a breaking API change.

/// A serializer that converts its input data to an `AnyLuaValue`.
pub struct LuaSerializer(PhantomData<()>);

impl LuaSerializer {
    /// Return a serializer that can serialize input data to an `AnyLuaValue`.
    pub fn new() -> LuaSerializer {
        LuaSerializer(PhantomData)
    }
}

impl Serializer for LuaSerializer {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;
    type SerializeSeq = LuaSerializeSeq;
    type SerializeTuple = LuaSerializeSeq;
    type SerializeTupleStruct = LuaSerializeSeq;
    type SerializeTupleVariant = LuaSerializeTupleVariant;
    type SerializeMap = LuaSerializeMap;
    type SerializeStruct = LuaSerializeMap;
    type SerializeStructVariant = LuaSerializeStructVariant;

    fn serialize_bool(self, v: bool) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaBoolean(v))
    }

    fn serialize_i8(self, v: i8) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_i16(self, v: i16) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_i32(self, v: i32) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_i64(self, v: i64) -> SerResult<AnyLuaValue> {
        if v as f64 as i64 != v {
            Err(serde::ser::Error::custom(
                "value cannot be losslessly represented as lua number (f64)"
            ))
        } else {
            Ok(AnyLuaValue::LuaNumber(v as f64))
        }
    }

    fn serialize_u8(self, v: u8) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_u16(self, v: u16) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_u32(self, v: u32) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_u64(self, v: u64) -> SerResult<AnyLuaValue> {
        if v as f64 as u64 != v {
            Err(serde::ser::Error::custom(
                "value cannot be losslessly represented as lua number (f64)"
            ))
        } else {
            Ok(AnyLuaValue::LuaNumber(v as f64))
        }
    }

    fn serialize_f32(self, v: f32) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v as f64))
    }

    fn serialize_f64(self, v: f64) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNumber(v))
    }

    fn serialize_char(self, v: char) -> SerResult<AnyLuaValue> {
        let mut result = String::new();
        result.push(v);
        Ok(AnyLuaValue::LuaString(result))
    }

    fn serialize_str(self, v: &str) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaString(v.to_owned()))
    }

    #[cfg(not(feature = "base64-bytes"))]
    fn serialize_bytes(self, v: &[u8]) -> SerResult<AnyLuaValue> {
        Err(LuaSerializeError::custom(
            "cannot serialize bytes; compile with 'base64-bytes'"
        ))
    }

    #[cfg(feature = "base64-bytes")]
    fn serialize_bytes(self, v: &[u8]) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaString(base64::encode(v)))
    }

    fn serialize_none(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNil)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> SerResult<AnyLuaValue>
        where T: serde::Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNil)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaNil)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str
    ) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaString(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T
    ) -> SerResult<AnyLuaValue>
        where T: serde::Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> SerResult<AnyLuaValue>
        where T: serde::Serialize
    {
        Ok(AnyLuaValue::LuaArray(vec![
            (AnyLuaValue::LuaString(variant.to_owned()), value.serialize(self)?)
        ]))
    }

    fn serialize_seq(self, len: Option<usize>) -> SerResult<LuaSerializeSeq> {
        Ok(LuaSerializeSeq(match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new()
        }))
    }

    fn serialize_tuple(self, len: usize) -> SerResult<LuaSerializeSeq> {
        Ok(LuaSerializeSeq(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize
    ) -> SerResult<LuaSerializeSeq> {
        Ok(LuaSerializeSeq(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> SerResult<LuaSerializeTupleVariant> {
        Ok(LuaSerializeTupleVariant(variant, LuaSerializeSeq(Vec::with_capacity(len))))
    }

    fn serialize_map(self, len: Option<usize>) -> SerResult<LuaSerializeMap> {
        Ok(LuaSerializeMap(match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new()
        }))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> SerResult<LuaSerializeMap> {
        Ok(LuaSerializeMap(Vec::with_capacity(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> SerResult<LuaSerializeStructVariant> {
        Ok(LuaSerializeStructVariant(variant, LuaSerializeMap(Vec::with_capacity(len))))
    }
}

pub struct LuaSerializeSeq(Vec<(AnyLuaValue, AnyLuaValue)>);

impl serde::ser::SerializeSeq for LuaSerializeSeq {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> SerResult<()>
        where T: Serialize
    {
        let index = (self.0.len() + 1) as f64;
        self.0.push((
            AnyLuaValue::LuaNumber(index),
            value.serialize(LuaSerializer::new())?
        ));
        Ok(())
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaArray(self.0))
    }
}

impl serde::ser::SerializeTuple for LuaSerializeSeq {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> SerResult<()>
        where T: Serialize
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for LuaSerializeSeq {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> SerResult<()>
        where T: Serialize
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        serde::ser::SerializeSeq::end(self)
    }
}

pub struct LuaSerializeTupleVariant(&'static str, LuaSerializeSeq);

impl serde::ser::SerializeTupleVariant for LuaSerializeTupleVariant {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> SerResult<()>
        where T: Serialize
    {
        serde::ser::SerializeSeq::serialize_element(&mut self.1, value)
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaArray(vec![(
            AnyLuaValue::LuaString(self.0.to_owned()),
            serde::ser::SerializeSeq::end(self.1)?
        )]))
    }
}

pub struct LuaSerializeMap(Vec<(AnyLuaValue, AnyLuaValue)>);

impl serde::ser::SerializeMap for LuaSerializeMap {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> SerResult<()>
        where T: Serialize
    {

        let key = key.serialize(LuaSerializer::new())?;
        match &key {
            &AnyLuaValue::LuaNumber(number) if number != number => return Err(
                serde::ser::Error::custom(&"unserializable key NaN")
            ),
            &AnyLuaValue::LuaNil => return Err(serde::ser::Error::custom(
                &"unserializable key nil"
            )),
            _ => {}
        }
        self.0.push((key, AnyLuaValue::LuaNil));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> SerResult<()>
        where T: Serialize
    {
        let len = self.0.len();
        self.0[len - 1].1 = value.serialize(LuaSerializer::new())?;
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V
    ) -> SerResult<()>
        where K: Serialize,
              V: Serialize
    {
        let key = key.serialize(LuaSerializer::new())?;
        match &key {
            &AnyLuaValue::LuaNumber(number) if number != number => return Err(
                serde::ser::Error::custom(&"unserializable key NaN")
            ),
            &AnyLuaValue::LuaNil => return Err(serde::ser::Error::custom(
                &"unserializable key nil"
            )),
            _ => {}
        }
        self.0.push((
            key,
            value.serialize(LuaSerializer::new())?
        ));
        Ok(())
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaArray(self.0))
    }
}

impl serde::ser::SerializeStruct for LuaSerializeMap {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> SerResult<()>
        where T: Serialize
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        serde::ser::SerializeMap::end(self)
    }
}

pub struct LuaSerializeStructVariant(&'static str, LuaSerializeMap);

impl serde::ser::SerializeStructVariant for LuaSerializeStructVariant {
    type Ok = AnyLuaValue;
    type Error = LuaSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> SerResult<()>
        where T: Serialize
    {
        serde::ser::SerializeMap::serialize_entry(&mut self.1, key, value)
    }

    fn end(self) -> SerResult<AnyLuaValue> {
        Ok(AnyLuaValue::LuaArray(vec![(
            AnyLuaValue::LuaString(self.0.to_owned()),
            serde::ser::SerializeMap::end(self.1)?
        )]))
    }
}

/// A result returned by lua serialization.
pub type SerResult<T> = Result<T, LuaSerializeError>;

/// An error returned by lua serialization.
#[derive(Debug, Clone)]
pub struct LuaSerializeError(String);

impl fmt::Display for LuaSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl error::Error for LuaSerializeError {
}

impl serde::ser::Error for LuaSerializeError {
    fn custom<T>(msg: T) -> Self
        where T: fmt::Display
    {
        LuaSerializeError(format!("{}", msg))
    }
}

#[cfg(test)]
mod tests {
    use std;
    use hlua;
    use serde::Serialize;

    use std::collections::BTreeMap;

    use ::to_lua;

    fn test<S: Serialize>(value: &S, test: &str, openlibs: bool) -> bool {
        let mut lua = hlua::Lua::new();
        if openlibs {
            lua.openlibs();
        }
        lua.execute::<()>("table = {}").unwrap();
        lua.set("value", to_lua(value).unwrap());
        let result = lua.execute::<bool>(test).unwrap();
        if !result {
            if !openlibs {
                lua.openlibs();
            }
            lua.execute::<()>("print(value)").unwrap();
        }
        result
    }

    fn test_eq<S: Serialize>(value: &S, test_value: &str) -> bool {
        test(value, &format!("return value == ({})", test_value), false)
    }

    fn test_result<S: Serialize>(value: &S) -> Result<(), ()> {
        let mut lua = hlua::Lua::new();
        lua.execute::<()>("table = {}").unwrap();
        {
            let mut table = lua.get::<hlua::LuaTable<_>, _>("table").unwrap();
            table.set("value", to_lua(value).map_err(|_|())?);
        }
        Ok(())
    }

    #[test]
    fn unit() {
        assert!(test_eq(&(), "nil"));
    }

    #[test]
    fn boolean() {
        assert!(test_eq(&true, "true"));
        assert!(test_eq(&false, "false"));
    }

    #[test]
    fn number() {
        assert!(test_eq(&1, "1"));
        assert!(test_eq(&1.5, "1.5"));
        assert!(test_eq(&-9, "-9"));
        assert!(test_eq(&std::f32::INFINITY, "1/0"));
        assert!(test_eq(&-std::f32::INFINITY, "-1/0"));
        assert!(test_eq(&1u8, "1"));
        assert!(test_eq(&1358u16, "1358"));
        assert!(test_eq(&13583953u32, "13583953"));
        assert!(test_eq(&135839530000000u64, "135839530000000"));

        assert!(test(
            &std::f32::NAN,
            "return type(value) == 'number' and value ~= value",
            true
        ));

        assert!(test_result(&std::u64::MAX).is_err());
        assert!(test_result(&std::i64::MAX).is_err());
        assert!(test_result(&(std::i64::MIN + 1)).is_err());
    }

    #[test]
    fn string() {
        assert!(test_eq(&"", "''"));
        assert!(test_eq(&"good morning", "'good morning'"));
        assert!(test_eq(&"κόσμε", "'κόσμε'"));
        assert!(test_eq(&"你好，世界", "'你好，世界'"));
        assert!(test_eq(&"💁", "'💁'"));
    }

    #[test]
    fn array() {
        assert!(test(&vec![] as &Vec<()>, "return #value == 0", false));
        assert!(test(&vec![(), (), ()] as &Vec<()>, "return #value == 0", false));

        assert!(test(
            &[1, 2, 3],
            "assert(#value == 3)
            for index, entry in ipairs(value) do
                if index ~= entry then
                    return false
                end
            end
            return true",
            true
        ));

        assert!(test(
            &[&[1, 2, 3, 4, 5], &[1, 2, 3, 4, 5]],
            "assert(#value == 2)
            for _, entry in ipairs(value) do
                assert(#entry == 5)
                for index, subentry in ipairs(entry) do
                    if index ~= subentry then
                        return false
                    end
                end
            end
            return true",
            true
        ));
    }

    #[test]
    fn map() {
        use std::iter::FromIterator;
        use serde::ser::{Serializer, SerializeMap};

        assert!(test(
            &BTreeMap::from_iter(vec![]) as &BTreeMap<(), ()>,
            "return #value == 0",
            false
        ));

        assert!(test(
            &BTreeMap::from_iter(vec![(1, ()), (2, ())]),
            "return #value == 0",
            false
        ));

        assert!(test(
            &BTreeMap::from_iter(vec![("hello", "world"), ("你好", "世界")]),
            "local count = 0
            for _, _ in pairs(value) do
                count = count + 1
            end
            return (
                count == 2 and
                value.hello == 'world' and
                value['你好'] == '世界')",
            true
        ));

        assert!(test_result(
            &BTreeMap::from_iter(vec![((), "world"), ((), "世界")])
        ).is_err());

        assert!(
            ::LuaSerializer::new()
                .serialize_map(Some(1)).unwrap()
                .serialize_entry(&std::f32::NAN, &"hello")
                .is_err()
        );
    }

    #[derive(Serialize)]
    struct Simple {
        x: f32,
        y: &'static str
    }

    #[derive(Serialize)]
    struct Nested {
        friend: Simple,
        list: &'static [u16]
    }

    #[test]
    fn structs() {
        assert!(test(
            &Simple {
                x: std::f32::NAN,
                y: "世界"
            },
            "local count = 0
            for _, _ in pairs(value) do
                count = count + 1
            end
            return (
                count == 2 and
                type(value.x) == 'number' and value.x ~= value.x and
                value.y == '世界')",
            true
        ));

        assert!(test(
            &Nested {
                friend: Simple {
                    x: 1.0,
                    y: "easy"
                },
                list: &[1, 2, 3]
            },
            "return (
                value.friend.x == 1.0 and
                value.friend.y == 'easy' and
                #value.list == 3 and
                value.list[1] == 1 and
                value.list[2] == 2 and
                value.list[3] == 3)",
            false
        ));

        assert!(test(
            &[Simple { x: 1.0, y: "first" }, Simple { x: 2.0, y: "second" }],
            "return (
                value[1].x == 1.0 and
                value[1].y == 'first' and
                value[2].x == 2.0 and
                value[2].y == 'second')",
            false
        ));
    }

    #[derive(Serialize)]
    enum Enum {
        UnitVariant,
        #[serde(rename = "renamed_unit_variant")]
        RenamedUnitVariant,
        TupleVariant(f32, f32),
        StructVariant {
            x: &'static str,
            y: &'static str
        }
    }

    #[test]
    fn enums() {
        assert!(test_eq(&Enum::UnitVariant, "'UnitVariant'"));
        assert!(test_eq(&Enum::RenamedUnitVariant, "'renamed_unit_variant'"));

        assert!(test(
            &Enum::TupleVariant(-4294.0, std::f32::INFINITY),
            "local count = 0
            for _, _ in pairs(value) do
                count = count + 1
            end
            return (
                count == 1 and
                #value.TupleVariant == 2 and
                value.TupleVariant[1] == -4294 and
                value.TupleVariant[2] == 1/0)",
            true
        ));

        assert!(test(
            &Enum::StructVariant {
                x: "hello",
                y: "世界"
            },
            "local count = 0
            for _, _ in pairs(value.StructVariant) do
                count = count + 1
            end
            local top_count = 0
            for _, _ in pairs(value) do
                top_count = top_count + 1
            end
            return (
                top_count == 1 and
                count == 2 and
                value.StructVariant.x == 'hello' and
                value.StructVariant.y == '世界')",
            true
        ));

        assert!(test(
            &[
                Enum::UnitVariant,
                Enum::TupleVariant(1.0, 2.0),
                Enum::StructVariant { x: "hiya", y: "globe" }
            ],
            "return (
                #value == 3 and
                value[1] == 'UnitVariant' and
                value[2].TupleVariant[1] == 1.0 and
                value[2].TupleVariant[2] == 2.0 and
                value[3].StructVariant.x == 'hiya' and
                value[3].StructVariant.y == 'globe')",
            false
        ));
    }

    #[test]
    fn bytes() {
        use serde_bytes::Bytes;
        assert!(test_eq(&Bytes::new(&[1, 2, 3, 4]), "'AQIDBA=='"));
        assert!(test_eq(
            &Bytes::new(&[91, 144, 255, 193, 22, 11, 52, 9, 3]),
            "'W5D/wRYLNAkD'"
        ));
    }

    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum InternallyTaggedEnum {
        TypeA {
            payload: f32
        },
        TypeB {
            payload: String
        },
    }

    #[derive(Serialize)]
    #[serde(untagged)]
    enum UntaggedEnum {
        TypeA(f32),
        TypeB(String)
    }

    #[test]
    fn enum_formats() {
        assert!(test(
            &InternallyTaggedEnum::TypeA { payload: 1.5 },
            "return (
                value.type == 'TypeA' and
                value.payload == 1.5)",
            false
        ));
        assert!(test(
            &InternallyTaggedEnum::TypeB { payload: "whoa!".to_string() },
            "return (
                value.type == 'TypeB' and
                value.payload == 'whoa!')",
            false
        ));
        assert!(test_eq(&UntaggedEnum::TypeA(1.5), "1.5"));
        assert!(test_eq(&UntaggedEnum::TypeB("yeehaw!".to_string()), "'yeehaw!'"));
    }
}

