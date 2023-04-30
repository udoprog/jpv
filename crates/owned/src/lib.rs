/// The missing `to_owned` implementor.
///
/// ```
/// # mod interior {
/// #[owned::owned]
/// #[derive(Clone, Debug)]
/// pub struct SourceLanguage<'a> {
///     #[owned(ty = String)]
///     pub text: &'a str,
///     #[owned(ty = Option<String>, with = self::option)]
///     pub lang: Option<&'a str>,
///     #[owned(copy)]
///     pub waseigo: bool,
///     #[owned(ty = Option<String>, with = self::option)]
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
///
/// #### `#[owned(borrowed(<attr>))]`
///
/// Apply the given attributes `<attr>` to a field, but only for the borrowed
/// variant.
///
/// ```
/// use serde::{Serialize, Deserialize};
///
/// #[owned::owned]
/// #[derive(Serialize, Deserialize)]
/// pub struct SourceLanguage<'a> {
///     #[owned(ty = Option<String>, borrowed(serde(borrow)))]
///     pub lang: Option<&'a str>,
/// }
/// ```
#[doc(inline)]
pub use owned_macros::owned;

mod borrow;
pub use self::borrow::Borrow;

mod to_owned;
pub use self::to_owned::ToOwned;

/// Convert the value into an owned variant.
///
/// This helper function is provided so that you don't have to have the
/// [`ToOwned`] trait in scope, and make it explicit when this crate is being
/// used since this conversion is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`ToOwned`][std::borrow::ToOwned].
pub fn to_owned<T>(value: T) -> T::Owned
where
    T: ToOwned,
{
    value.to_owned()
}

/// Borrow the given value.
///
/// This helper function is provided so that you don't have to have the [`Borrow`]
/// trait in scope, and make it explicit when this crate is being used since
/// "borrowing" is not a cheap operation in this crate.
///
/// This also prevents conflicts with the built-in
/// [`Borrow`][std::borrow::Borrow].
pub fn borrow<T>(value: &T) -> T::Target<'_>
where
    T: ?Sized + Borrow,
{
    value.borrow()
}
