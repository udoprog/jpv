//! The missing compositional borrowing for Rust.
//!
//! This crate provides an attribute macro which helps you achieve compositional
//! borrowing. Roughly this means that you can convert a struct which has
//! lifetimes into ones which does not and vice versa.
//!
//! > **Note:** See the [`#[owned::owned]`][owned] attribute for more
//! > documentation.
//!
//! ```
//! #[owned::owned]
//! #[derive(Debug, PartialEq, Eq)]
//! struct Word<'a> {
//!     #[owned(ty = String)]
//!     text: &'a str,
//!     #[owned(ty = Option<String>)]
//!     lang: Option<&'a str>,
//! }
//!
//! let text = String::from("Hello World");
//! let lang = Some(String::from("eng"));
//!
//! let word = Word {
//!     text: "hello",
//!     lang: lang.as_deref(),
//! };
//!
//! let owned_word: OwnedWord = owned::to_owned(&word);
//! assert_eq!(&owned_word.text, word.text);
//! assert_eq!(owned_word.lang.as_deref(), word.lang);
//!
//! let word2: Word<'_> = owned::borrow(&owned_word);
//! assert_eq!(word2, word);
//! ```
//!
//! <br>
//!
//! ## Why can't we use [`std::borrow::ToOwned`][std-to-owned] and [`std::borrow::Borrow`][std-borrow]?
//!
//! Rust comes with two sibling traits which both are responsible for converting
//! something to an owned and a borrowed variant: [`ToOwned`][std-to-owned] and
//! [`Borrow`][std-borrow].
//!
//! These convert a type to a *borrowed* value to an owned one, let's think
//! about it from a broader perspective: How to we convert a type which *has
//! lifetimes*, to one which *does not*?
//!
//! ```
//! struct Word<'a> {
//!     text: &'a str,
//!     lang: Option<&'a str>,
//! }
//! ```
//!
//! Let's try to implement `ToOwned` for this type.
//!
//! ```compile_fail
//! # struct Word<'a> { text: &'a str, lang: Option<&'a str> }
//! struct OwnedWord {
//!     text: String,
//!     lang: Option<String>,
//! }
//!
//! impl ToOwned for Word<'_> {
//!     type Owned = OwnedWord;
//!
//!     #[inline]
//!     fn to_owned(&self) -> OwnedWord {
//!         OwnedWord {
//!             text: self.text.to_owned(),
//!             lang: self.lang.map(ToOwned::to_owned),
//!         }
//!     }
//! }
//! ```
//!
//! ```text
//! error[E0277]: the trait bound `OwnedWord: std::borrow::Borrow<Word<'_>>` is not satisfied
//!   --> src/lib.rs:27:18
//!    |
//! 11 |     type Owned = OwnedWord;
//!    |                  ^^^^^^^^^ the trait `std::borrow::Borrow<Word<'_>>` is not implemented for `OwnedWord`
//!    |
//! note: required by a bound in `std::borrow::ToOwned::Owned`
//!   --> alloc/src/borrow.rs:41:17
//!    |
//! 41 |     type Owned: Borrow<Self>;
//!    |                 ^^^^^^^^^^^^ required by this bound in `ToOwned::Owned`
//! ```
//!
//! This happens because [`ToOwned`][std-to-owned] is a symmetric trait, which
//! requires that the resulting `Owned` type can be borrowed back into the type
//! being converted.
//!
//! So the first task is to define a new [`ToOwned`] trait which does not
//! require the produced value to be [`Borrow`][std-borrow].
//!
//! Simple enough, but what if we need to go *the other* way?
//!
//! The [`Borrow`][std-borrow] trait as defined faces an issue which can't be
//! easily addressed. The `borrow` method returns *a reference* to the borrowed
//! type.
//!
//! ```
//! pub trait Borrow<Borrowed: ?Sized> {
//!     fn borrow(&self) -> &Borrowed;
//! }
//! ```
//!
//! This means that there is no way to implement `Borrow<Word<'a>>` because it
//! required that we return a reference which doesn't outlive `'a`, something
//! that can't be satisfied because we don't hold a reference to `Word<'a>`.
//!
//! ```compile_fail
//! # use std::borrow::Borrow;
//! # struct Word<'a> { text: &'a str, lang: Option<&'a str> }
//! # struct OwnedWord { text: String, lang: Option<String> }
//! impl<'a> Borrow<Word<'a>> for OwnedWord {
//!     fn borrow(&self) -> &Word<'a> {
//!         &Word {
//!            text: self.text.as_str(),
//!            lang: self.lang.as_ref().map(String::as_str),
//!         }
//!     }
//! }
//! ```
//!
//! ```text
//! error: lifetime may not live long enough
//!   --> src/lib.rs:83:9
//!    |
//! 6  |   impl<'a> Borrow<Word<'a>> for OwnedWord {
//!    |        -- lifetime `'a` defined here
//! 7  |       fn borrow(&self) -> &Word<'a> {
//!    |                 - let's call the lifetime of this reference `'1`
//! 8  | /         &Word {
//! 9  | |            text: self.text.as_str(),
//! 10 | |            lang: self.lang.as_ref().map(String::as_str),
//! 11 | |         }
//!    | |_________^ associated function was supposed to return data with lifetime `'a` but it is returning data with lifetime `'1`
//! ```
//!
//! The solution this crate provides is to define a new [`Borrow`] trait which
//! makes use of [generic associated types]. This allows it to structurally
//! decompose a borrowed value.
//!
//! ```rust
//! pub trait Borrow {
//!     type Target<'a>
//!     where
//!         Self: 'a;
//!
//!     fn borrow(&self) -> Self::Target<'_>;
//! }
//! ```
//!
//! Now we can implement it for `OwnedWord`:
//!
//! ```
//! # use owned::Borrow;
//! # struct Word<'a> { text: &'a str, lang: Option<&'a str> }
//! # struct OwnedWord { text: String, lang: Option<String> }
//! impl Borrow for OwnedWord {
//!     type Target<'a> = Word<'a>;
//!
//!     fn borrow(&self) -> Self::Target<'_> {
//!         Word {
//!            text: self.text.as_str(),
//!            lang: self.lang.as_ref().map(String::as_str),
//!         }
//!     }
//! }
//! ```
//!
//! The catch here is that `Borrow` can only be implemented once for each time,
//! compared to [`Borrow<T>`][std-borrow]. But for our purposes this is fine.
//! This crate is primarily intended to work with two symmetrical types.
//!
//! [`ToOwned`]: https://docs.rs/owned
//! [generic associated types]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
//! [owned]: https://docs.rs/owned
//! [std-borrow]: std::borrow::Borrow
//! [std-to-owned]: std::borrow::ToOwned

/// Automatically implement owned variants of borrowed structs and implement
/// [`ToOwned`] and [`Borrow`].
///
/// In order to work as intended, `#[owned]` must be used *before* any
/// attributes that you want it to capture, such as derives.
///
/// ```
/// use serde::Serialize;
///
/// #[owned::owned]
/// #[derive(Serialize)]
/// pub struct Word<'a> {
///     #[owned(ty = String)]
///     lang: &'a str,
/// }
///
/// # fn implements_serialize<T: serde::Serialize>() {}
/// implements_serialize::<Word<'_>>();
/// implements_serialize::<OwnedWord>();
/// ```
///
/// If we use the wrong order, `Serialize` won't be seen and that derive won't
/// be transferred to `OwnedWord`.
///
/// ```compile_fail
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// #[owned::owned]
/// pub struct Word<'a> {
///     #[owned(ty = String)]
///     lang: &'a str,
/// }
///
/// # fn implements_serialize<T: serde::Serialize>() {}
/// implements_serialize::<Word<'_>>();
/// implements_serialize::<OwnedWord>();
/// ```
///
/// ```text
/// 15 | implements_serialize::<OwnedWord>();
///    |                        ^^^^^^^^^ the trait `Serialize` is not implemented for `OwnedWord`
/// ```
///
/// ## Container attributes
///
/// #### `#[owned(prefix = <ident>)]`
///
/// This allows you to pick the prefix to use for the generated type. By default
/// this is `Owned`.
///
/// ## Field attributes
///
/// Field attributes are attributes which apply to fields, such as the fields in
/// a struct.
///
/// <br>
///
/// #### `#[owned(to_owned = <path>)]`
///
/// Specifies a path to use when making a field owned. By default this is
/// `::owned::ToOwned::to_owned` unless `#[owned(copy)]` is specified.
///
/// ```
/// #[owned::owned]
/// #[derive(Clone, Debug)]
/// pub struct Word<'a> {
///     #[owned(ty = Option<String>, to_owned_with = option_to_owned)]
///     lang: Option<&'a str>,
/// }
///
/// #[inline]
/// pub(crate) fn option_to_owned(option: &Option<&str>) -> Option<String> {
///     option.map(ToOwned::to_owned)
/// }
/// ```
///
/// #### `#[owned(borrow_with = <path>)]`
///
/// Specifies a path to use when borrowing a field. By default this is
/// `::owned::Borrowed::borrow` unless `#[owned(copy)]` is specified.
///
/// ```
/// #[owned::owned]
/// #[derive(Clone, Debug)]
/// pub struct Word<'a> {
///     #[owned(ty = Option<String>, borrow_with = option_borrow)]
///     lang: Option<&'a str>,
///     // Note that the above works the same as `Option::as_deref`.
///     #[owned(ty = Option<String>, borrow_with = Option::as_deref)]
///     lang2: Option<&'a str>,
/// }
///
/// #[inline]
/// pub(crate) fn option_borrow(option: &Option<String>) -> Option<&str> {
///     option.as_deref()
/// }
/// ```
///
/// #### `#[owned(with = <path>)]`
///
/// Specifies a path to use when calling `to_owned` and `borrow` on a field.
///
/// The sets `to_owned` to `<path>::to_owned`, and `borrow` to `<path>::borrow`.
/// By default these are otherwise `::owned::ToOwned::to_owned` and
/// `::owned::Borrowed::borrow` unless `#[owned(copy)]` is specified.
///
/// ```
/// # mod interior {
/// #[owned::owned]
/// #[derive(Clone, Debug)]
/// pub struct Word<'a> {
///     #[owned(ty = String)]
///     text: &'a str,
///     #[owned(ty = Option<String>, with = self::option)]
///     lang: Option<&'a str>,
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
/// <br>
///
/// #### `#[owned(copy)]`
///
/// Indicates that the field type is `Copy`, if this is set then the value is
/// not cloned when the type is converted to and from its owned variant.
///
/// ```
/// #[owned::owned]
/// pub struct Word<'a> {
///     #[owned(ty = String)]
///     text: &'a str,
///     #[owned(copy)]
///     teineigo: bool,
/// }
/// ```
///
/// <br>
///
/// #### `#[owned(borrowed(<meta>))]`
///
/// Apply the given attribute `<meta>` to a field, but only for the borrowed
/// variant. This allows certain attributes that are only needed for the
/// borrowed variant to be implemented, such as `#[serde(borrow)]`.
///
/// ```
/// use serde::{Serialize, Deserialize};
///
/// #[owned::owned]
/// #[derive(Serialize, Deserialize)]
/// pub struct Word<'a> {
///     #[owned(ty = Option<String>, borrowed(serde(borrow)))]
///     lang: Option<&'a str>,
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
