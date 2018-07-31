# serde-hlua

An easy way to transparently create serializable types that are fully legible
from lua as plain lua types.

Usage
---

The `to_lua` and `from_lua` functions are the easiest way to serialize any
datatype that implements serde's serialize and deserialize traits,
respectively.

    extern crate hlua;
    extern crate serde;
    #[macro_use] extern crate serde_derive;
    extern crate serde_hlua;
    
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Point {
        x: f32,
        y: f32
    }
    
    fn main() {
        let mut lua = hlua::Lua::new();
        
        let my_point = Point { x: 3.0, y: 4.0 };
        lua.set("my_value", serde_hlua::to_lua(&my_point).unwrap());
        
        let retrieved_point: Point = serde_hlua::from_lua(
            lua.get::<hlua::AnyLuaValue, _>("my_value").unwrap()
        ).unwrap();
        
        assert_eq!(my_point, retrieved_point);
    }

Known limitations
---

 * Mappings and sequences to unit structs are not surjective. In other
   words, lua will convert all of `[]`, `[()]`, and `[(), ()]` to the
   empty array `{}`. This is because lua considers setting a value to
   nil to be equivalent to erasing it from a table, and so the table
   `{a = nil, b = nil}` is the same as the table `{}`. Because of this,
   round-tripping a structure that uses unit structs as values will be
   lossy.

   However, this is not always an issue; Marking a unit-struct field
   with `#[serde(default)]` will allow serde to fill it in even when
   it's not present, so deserializing `struct {a: (), b: ()}` will
   succeed when given the lua table `{}`.

   Unit enum variants are also encoded losslessly, as they are encoded
   as the name of the variant as a string.

 * Integer values are only serialized and deserialized if they can do
   so losslessly. `std::i64::MIN` can be losslessly encoded, but
   `std::i64::MIN + 1` cannot, as it is rounded to a different value.

   `f32` values are always encoded into `f64`, as otherwise `f64`
   values with too many significant digits (such as `1/3`) would not
   encode. They are cast using rust's `as` operator.
