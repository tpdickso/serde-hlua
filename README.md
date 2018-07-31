# serde-hlua

An easy way to transparently create serializable types that are fully legible
from lua as plain lua types.

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
