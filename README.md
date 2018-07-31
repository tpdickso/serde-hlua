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
