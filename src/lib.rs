
//! Implementation of serialization and deserialization for lua structures.
//! 
//! Usage
//! ---
//!
//! The `to_lua` and `from_lua` functions are the easiest way to serialize any
//! datatype that implements serde's serialize and deserialize traits,
//! respectively.
//!
//! ```rust
//! extern crate hlua;
//! extern crate serde;
//! #[macro_use] extern crate serde_derive;
//! extern crate serde_hlua;
//!
//! #[derive(Debug, Deserialize, Serialize, PartialEq)]
//! struct Point {
//!     x: f32,
//!     y: f32
//! }
//!
//! fn main() {
//!     let mut lua = hlua::Lua::new();
//!
//!     let my_point = Point { x: 3.0, y: 4.0 };
//!     lua.set("my_value", serde_hlua::to_lua(&my_point).unwrap());
//!
//!     let retrieved_point: Point = serde_hlua::from_lua(
//!         lua.get::<hlua::AnyLuaValue, _>("my_value").unwrap()
//!     ).unwrap();
//!
//!     assert_eq!(my_point, retrieved_point);
//! }
//! ```
//! 
//! The serialization implemented by this crate doesn't use lua's userdata
//! system; all types are serialized to plain lua types.
//! 
//! ```rust
//! # extern crate hlua;
//! # extern crate serde;
//! # #[macro_use] extern crate serde_derive;
//! # extern crate serde_hlua;
//! #
//! # #[derive(Debug, Deserialize, Serialize, PartialEq)]
//! # struct Point {
//! #     x: f32,
//! #     y: f32
//! # }
//! #
//! # fn main() {
//! #     let mut lua = hlua::Lua::new();
//! let lua_point: Point = serde_hlua::from_lua(lua.execute::<hlua::AnyLuaValue>("
//!     return { x = 3.5, y = 7 }
//! ").unwrap()).unwrap();
//!
//! assert_eq!(lua_point, Point { x: 3.5, y: 7.0 });
//! # }
//! ```
//! 
//! Typechecking is performed automatically, so an invalid lua value can't
//! deserialize to a type that doesn't support it.
//! 
//! ```rust
//! # extern crate hlua;
//! # extern crate serde;
//! # #[macro_use] extern crate serde_derive;
//! # extern crate serde_hlua;
//! #
//! # #[derive(Debug, Deserialize, Serialize, PartialEq)]
//! # struct Point {
//! #     x: f32,
//! #     y: f32
//! # }
//! #
//! # fn main() {
//! #     let mut lua = hlua::Lua::new();
//! assert!(serde_hlua::from_lua::<Point>(lua.execute::<hlua::AnyLuaValue>("
//!     return { x = 3.5, y = 'wrong type!' }
//! ").unwrap()).is_err());
//! # }
//! ```
//! 
//! Known limitations
//! ---
//!
//! * Mappings and sequences to unit structs are not injective.
//!   Lua will convert all of `[]`, `[()]`, and `[(), ()]` to the
//!   empty array `{}`. This is because lua considers setting a value to
//!   nil to be equivalent to erasing it from a table, and so the table
//!   `{a = nil, b = nil}` is the same as the table `{}`. Because of this,
//!   round-tripping a structure that uses unit structs as values will be
//!   lossy.
//!
//!   However, this is not always an issue; Marking a unit-struct field
//!   with `#[serde(default)]` will allow serde to fill it in even when
//!   it's not present, so deserializing `struct {a: (), b: ()}` will
//!   succeed when given the lua table `{}`.
//!
//!   Unit enum variants are also encoded losslessly, as they are encoded
//!   as the name of the variant as a string.
//!
//! * Integer values are only serialized and deserialized if they can do
//!   so losslessly. `std::i64::MIN` can be losslessly encoded, but
//!   `std::i64::MIN + 1` cannot, as it is rounded to a different value.
//!
//!   `f32` values are always encoded into `f64`, as otherwise `f64`
//!   values with too many significant digits (such as `1/3`) would not
//!   encode. They are cast using rust's `as` operator.

#[cfg(feature = "base64-bytes")]
extern crate base64;
extern crate hlua;
extern crate serde;
#[cfg(test)]
extern crate serde_bytes;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod de;
pub mod ser;
pub mod macros;

pub use de::LuaDeserializer;
pub use ser::LuaSerializer;

/// Convert a value to an `AnyLuaValue`.
pub fn to_lua<T: ?Sized>(value: &T) -> ser::SerResult<hlua::AnyLuaValue>
    where T: serde::Serialize
{
    value.serialize(LuaSerializer::new())
}

/// Convert a value from an `AnyLuaValue`.
pub fn from_lua<'de, T>(value: hlua::AnyLuaValue) -> de::DeResult<T>
    where T: serde::Deserialize<'de>
{
    T::deserialize(LuaDeserializer::new(value))
}

/// Implements `Push` for any type which is `Serialize`.
///
/// This makes it easy to call lua functions with rust structures:
/// 
/// ```rust
/// extern crate hlua;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
/// extern crate serde_hlua;
/// 
/// use serde_hlua::SerdeLuaPush;
/// 
/// #[derive(Serialize)]
/// struct Point {
///     x: f32,
///     y: f32
/// }
/// 
/// fn main() {
///     let mut lua = hlua::Lua::new();
///     lua.execute::<()>(
///         "dot = function(a, b) return a.x * b.x + a.y * b.y end"
///     ).unwrap();
///     let mut dot: hlua::LuaFunction<_> = lua.get("dot").unwrap();
///     let result: f32 = dot.call_with_args((
///         SerdeLuaPush(Point { x: 1.0, y: 1.0 }),
///         SerdeLuaPush(Point { x: 2.0, y: 3.0 })
///     )).unwrap();
///     assert_eq!(result, 5.0f32);
/// }
/// ```
///
/// This can be made even more ergonomic by implementing `Push` for your
/// type in terms of `SerdeLuaPush`. The macro `serde_lua_impl_read!` does
/// this automatically.
pub struct SerdeLuaPush<T: serde::Serialize>(pub T);

impl<'lua, L, T> hlua::Push<L> for SerdeLuaPush<T>
    where L: hlua::AsMutLua<'lua>,
          T: serde::Serialize
{
    type Err = ser::LuaSerializeError;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (ser::LuaSerializeError, L)> {
        match to_lua(&self.0) {
            Ok(any) => hlua::Push::push_to_lua(any, lua)
                .map_err(|_| unreachable!()),
            Err(error) => Err((error, lua))
        }
    }
}

impl<'lua, L, T> hlua::PushOne<L> for SerdeLuaPush<T>
    where L: hlua::AsMutLua<'lua>,
          T: serde::Serialize
{
}

/// Implements `LuaRead` for any type which is `Deserialize`.
///
/// This makes it easy to call rust functions from lua:
/// 
/// ```rust
/// extern crate hlua;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
/// #[macro_use] extern crate serde_hlua;
///
/// use serde_hlua::SerdeLuaRead;
///
/// #[derive(Deserialize)]
/// struct Point {
///     x: f32,
///     y: f32
/// }
///
/// fn dot(a: Point, b: Point) -> f32 {
///     a.x * b.x + a.y * b.y
/// }
///
/// fn main() {
///     let mut lua = hlua::Lua::new();
///     lua.set("dot", hlua::function2(|a: SerdeLuaRead<Point>, b: SerdeLuaRead<Point>| {
///         dot(a.0, b.0)
///     }));
///     println!("{:?}", lua.execute::<f32>(
///             "return dot({ x = 1.0, y = 1.0 }, { x = 2.0, y = 3.0 })"
///         ));
///     assert_eq!(
///         lua.execute::<f32>(
///             "return dot({ x = 1.0, y = 1.0 }, { x = 2.0, y = 3.0 })"
///         ).unwrap(),
///         5.0
///     );
/// }
/// ```
///
/// This can be made even more ergonomic by implementing `Push` for your
/// type in terms of `SerdeLuaPush`. The macro `serde_lua_impl_read!` does
/// this automatically.
pub struct SerdeLuaRead<T>(pub T)
    where T: for<'de> serde::Deserialize<'de>;

impl<'lua, L, T> hlua::LuaRead<L> for SerdeLuaRead<T>
    where L: hlua::AsMutLua<'lua>,
          T: for<'de> serde::Deserialize<'de>
{
    #[inline]
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<Self, L> {
        {
            let any_maybe = hlua::AnyLuaValue::lua_read_at_position(
                &mut lua as &mut hlua::AsMutLua<'lua>,
                index
            );
            match any_maybe {
                Ok(any) => match from_lua::<T>(any) {
                    Ok(value) => return Ok(SerdeLuaRead(value)),
                    Err(_) => {}
                },
                Err(_) => {}
            };
        }
        Err(lua)
    }
}
