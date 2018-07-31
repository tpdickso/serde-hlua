
/// Public re-exports of hlua traits, to enable the macros to work. Do not
/// access these traits through this module; access them through the
/// `hlua` crate instead.
pub mod hlua {
    pub use hlua::{Push, PushOne, PushGuard, LuaRead, AsMutLua};
}

/// Writes a `Push` impl and a `PushOne` impl for any type which is
/// `Serialize`.
///
/// ```rust
/// extern crate hlua;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
/// #[macro_use] extern crate serde_hlua;
///
/// #[derive(Serialize)]
/// struct Point {
///     x: f32,
///     y: f32
/// }
///
/// serde_hlua_impl_push!(Point);
///
/// fn main() {
///     let mut lua = hlua::Lua::new();
///     lua.execute::<()>(
///         "dot = function(a, b) return a.x * b.x + a.y * b.y end"
///     ).unwrap();
///     let mut dot: hlua::LuaFunction<_> = lua.get("dot").unwrap();
///     let result: f32 = dot.call_with_args((
///         Point { x: 1.0, y: 1.0 },
///         Point { x: 2.0, y: 3.0 }
///     )).unwrap();
///     assert_eq!(result, 5.0f32);
/// }
/// ```
#[macro_export]
macro_rules! serde_hlua_impl_push {
    ($type: ty) => {
        impl<'lua, L> $crate::macros::hlua::Push<L> for $type
            where L: $crate::macros::hlua::AsMutLua<'lua>
        {
            type Err = $crate::ser::LuaSerializeError;

            #[inline]
            fn push_to_lua(
                self,
                lua: L
            ) -> Result<
                $crate::macros::hlua::PushGuard<L>,
                ($crate::ser::LuaSerializeError, L)
            > {
                $crate::SerdeLuaPush(self).push_to_lua(lua)
            }
        }

        impl<'lua, L> $crate::macros::hlua::PushOne<L> for $type
            where L: $crate::macros::hlua::AsMutLua<'lua>
        {
        }
    }
}

/// Writes a `LuaRead` impl for any type which is `Deserialize`.
///
/// ```rust
/// extern crate hlua;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
/// #[macro_use] extern crate serde_hlua;
///
/// #[derive(Deserialize)]
/// struct Point {
///     x: f32,
///     y: f32
/// }
///
/// serde_hlua_impl_read!(Point);
/// 
/// fn dot(a: Point, b: Point) -> f32 {
///     a.x * b.x + a.y * b.y
/// }
///
/// fn main() {
///     let mut lua = hlua::Lua::new();
///     lua.set("dot", hlua::function2(dot));
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
#[macro_export]
macro_rules! serde_hlua_impl_read {
    ($type: ty) => {
        impl<'lua, L> $crate::macros::hlua::LuaRead<L> for $type
            where L: $crate::macros::hlua::AsMutLua<'lua>
        {
            #[inline]
            fn lua_read_at_position(mut lua: L, index: i32) -> Result<Self, L> {
                $crate::SerdeLuaRead::lua_read_at_position(lua, index)
                    .map(|wrapper| wrapper.0)
            }
        }
    }
}
