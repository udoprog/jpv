/// The missing `to_owned` implementor.
///
/// ```
/// # mod interior {
/// #[derive(Clone, Debug)]
/// #[owned::to_owned]
/// pub struct SourceLanguage<'a> {
///     #[to_owned(ty = String)]
///     pub text: &'a str,
///     #[to_owned(ty = Option<String>, with = self::option)]
///     pub lang: Option<&'a str>,
///     #[to_owned(copy)]
///     pub waseigo: bool,
///     #[to_owned(ty = Option<String>, with = self::option)]
///     pub ty: Option<&'a str>,
/// }
///
/// pub(crate) mod option {
///     use owned::{Borrow, ToOwned};
///
///     #[inline]
///     pub(crate) fn borrow<T>(this: &Option<T>) -> Option<T::Target<'_>>
///     where
///         T: Borrow,
///     {
///         match this {
///             Some(some) => Some(some.borrow()),
///             None => None,
///         }
///     }
///
///     #[inline]
///     pub(crate) fn to_owned<T>(option: &Option<T>) -> Option<T::Owned>
///     where
///         T: ToOwned,
///     {
///         option.as_ref().map(ToOwned::to_owned)
///     }
/// }
/// # }
/// ```
///
/// ## Field attributes
///
/// ## `#[owned(with = <path>)]`
///
/// Specifies a path to use when calling `to_owned` and `borrow` on a field.
///
/// The sets `to_owned` to `<path>::to_owned`, and `borrow` to `<path>::borrow`.
///
/// #### `#[owned(copy)]`
///
/// Indicates that the type is `Copy`, if this is set then the value is not
/// cloned.
#[doc(inline)]
pub use owned_macros::to_owned;

mod borrow;
pub use self::borrow::Borrow;

mod to_owned;
pub use self::to_owned::ToOwned;
