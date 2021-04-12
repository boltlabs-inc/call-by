//! # Calling Convention Polymorphism in Rust
//!
//! To parameterize a function by calling convention, we can specify that it takes some `T:
//! By<'a, Convention>`, and say that its input is of type `<T as By<'a,
//! Convention>>::Type`. This is essentially a defunctionalization of Rust's reference operators.

//! This trick can be used to permit the *implementor* of a trait to pick the calling convention for
//! a value passed into (or out of) a function defined in that trait, rather than this being
//! hardcoded in the trait definition.

//! ## Examples

//! For instance, say we wanted to define an abstraction for channels that can send values. Imagine,
//! however, that some channels might need to take ownership of the values they send, while others
//! might serialize values given only a reference to that value. In order to unify these two notions
//! into one trait, we can parameterize over the calling convention for the input value:

//! ```rust
//! use call_by::{By, Convention};

//! trait Sender<'a, T>
//! where
//!     T: By<'a, Self::Convention>,
//! {
//!     type Convention: Convention;
//!     fn send(&self, value: <T as By<'a, Self::Convention>>::Type);
//! }
//! ```

//! Implementers of the `Sender` trait can choose whether the associated type `Convention` should be
//! `Val`, `Ref`, or `Mut`, which toggles the result of `<T as By<'a, Self::Convention>>::Type`
//! between `T`, `&'a T`, and `&'a mut T`, respectively. Meanwhile, callers of the `send` method on
//! concretely known types don't need to specify the calling convention; the type-level function
//! determines what type they need to pass as the argument to `send`, and type errors are reported
//! in reference to that concrete type if it is known at the call site.
//! <!-- snip -->

/// There are three fundamental ways to pass a `T` as input or return a `T` as output: by [`Val`]ue,
/// by shared immutable [`Ref`]erence, and by unique [`Mut`]able reference.
///
/// This is a sealed trait, implemented for all three of these conventions.
pub trait Convention: sealed::Convention + Sized {
    const TOKEN: Self;
}

impl Convention for Val {
    const TOKEN: Self = Val;
}

impl Convention for Ref {
    const TOKEN: Self = Ref;
}

impl Convention for Mut {
    const TOKEN: Self = Mut;
}

/// To get the type of `T` via calling convention `Convention`, write `<T as By<'a,
/// Convention>>::Type`.
pub trait By<'a, C: Convention> {
    /// The type of `Self` when called by `Convention`.
    type Type;

    /// Copy a thing of unknown calling convention, returning an owned value.
    fn copy(this: Self::Type) -> Self
    where
        Self: Copy;

    /// Clone a thing of unknown calling convention, returning an owned value.
    fn clone(this: Self::Type) -> Self
    where
        Self: Clone;
}

/// Taking a `T` by [`Val`]ue means taking a `T` as input to or output from a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Val;

impl<'a, T> By<'a, Val> for T {
    type Type = T;

    fn copy(this: Self::Type) -> Self
    where
        Self: Copy,
    {
        this
    }

    fn clone(this: Self::Type) -> Self
    where
        Self: Clone,
    {
        this
    }
}

/// Taking a `T` by [`Ref`]erence means taking `&'a T` as input to or output from a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Ref;

impl<'a, T: 'a + ?Sized> By<'a, Ref> for T {
    type Type = &'a T;

    fn copy(this: Self::Type) -> Self
    where
        Self: Copy,
    {
        *this
    }

    fn clone(this: Self::Type) -> Self
    where
        Self: Clone,
    {
        this.clone()
    }
}

/// Taking a `T` by [`Mut`]able reference means taking `&'a mut T` as input to or output from a
/// function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Mut;

impl<'a, T: 'a + ?Sized> By<'a, Mut> for T {
    type Type = &'a mut T;

    fn copy(this: Self::Type) -> Self
    where
        Self: Copy,
    {
        *this
    }

    fn clone(this: Self::Type) -> Self
    where
        Self: Clone,
    {
        this.clone()
    }
}

/// Convert between different calling conventions.
///
/// Only some conversions are sensible in Rust, due to the ownership system. These are the valid
/// conversions, with the constraints on the underlying type `T` noted:
///
/// | Can I convert... | ... to [`Val`] (`T`)  | ... to [`Ref`] (`&'a T`) | ... to [`Mut`] (`&'a mut T`) |
/// | :--------------------------------- | :------------------ | :------ | :------ |
/// | **from [`Val`] (`T`) ...**         | (valid for all `T`) | ❌*     | ❌*     |
/// | **from [`Ref`] (`&'a T`) ...**     | `T: 'a +` [`Clone`] | `T: 'a` | ❌**    |
/// | **from [`Mut`] (`&'a mut T`) ...** | `T: 'a +` [`Clone`] | `T: 'a` | `T: 'a` |
///
/// > \* Impossible because references can't outlive the data they borrow.
/// >
/// > \** Impossible because potentially-aliased data can't be mutably referenced.
pub trait Convert<'a, From: Convention, To: Convention>
where
    Self: By<'a, To> + By<'a, From>,
{
    /// Convert from one calling convention to another.
    ///
    /// Because of the generic parameters on the trait, this often requires rather explicit type
    /// annotations.
    ///
    /// # Examples
    ///
    /// ```
    /// use call_by::*;
    ///
    /// let a: u8 = <u8 as Convert<Val, Val>>::convert(1);
    /// let b: u8 = <u8 as Convert<Ref, Val>>::convert(&2);      // implicit clone
    /// let c: u8 = <u8 as Convert<Mut, Val>>::convert(&mut 3);  // implicit clone
    ///
    /// let d: &u8 = <u8 as Convert<Ref, Ref>>::convert(&4);
    /// let e: &u8 = <u8 as Convert<Mut, Ref>>::convert(&mut 5);
    ///
    /// let b: &mut u8 = <u8 as Convert<Mut, Mut>>::convert(&mut 6);
    /// ```
    #[allow(clippy::wrong_self_convention)]
    fn convert(from: <Self as By<'a, From>>::Type) -> <Self as By<'a, To>>::Type;
}

impl<'a, T> Convert<'a, Val, Val> for T {
    fn convert(from: T) -> T {
        from
    }
}

impl<'a, T: 'a + Clone> Convert<'a, Ref, Val> for T {
    fn convert(from: &T) -> T {
        from.clone()
    }
}

impl<'a, T: 'a + Clone> Convert<'a, Mut, Val> for T {
    fn convert(from: &mut T) -> T {
        Clone::clone(from)
    }
}

impl<'a, T: 'a> Convert<'a, Ref, Ref> for T {
    fn convert(from: &T) -> &T {
        from
    }
}

impl<'a, T: 'a> Convert<'a, Mut, Ref> for T {
    fn convert(from: &mut T) -> &T {
        &*from
    }
}

impl<'a, T: 'a> Convert<'a, Mut, Mut> for T {
    fn convert(from: &mut T) -> &mut T {
        from
    }
}

/// The generalization of [`Into`], [`AsRef`], and [`AsMut`]: in a calling-convention polymorphic
/// context, this trait allows you to invoke the appropriate conversion method depending on the
/// applicable calling convention.
///
/// # Examples
///
/// ```
/// use call_by::*;
///
/// fn do_something<'a, T, S, C>(input: <S as By<'a, C>>::Type)
/// where
///     T: By<'a, C>,
///     S: By<'a, C> + As<'a, C, T>,
///     C: Convention,
/// {
///     let t: <T as By<'a, C>>::Type = S::as_convention(input);
///     // ... do something with `t` ...
/// }
/// ```
pub trait As<'a, C: Convention, T: By<'a, C>>: By<'a, C> {
    #[allow(clippy::wrong_self_convention)]
    fn as_convention(this: <Self as By<'a, C>>::Type) -> <T as By<'a, C>>::Type;
}

impl<'a, T, S> As<'a, Val, T> for S
where
    S: Into<T>,
{
    fn as_convention(this: S) -> T {
        this.into()
    }
}

impl<'a, T: 'a, S: 'a> As<'a, Ref, T> for S
where
    S: AsRef<T>,
{
    fn as_convention(this: &S) -> &T {
        this.as_ref()
    }
}

impl<'a, T: 'a, S: 'a> As<'a, Mut, T> for S
where
    S: AsMut<T>,
{
    fn as_convention(this: &mut S) -> &mut T {
        this.as_mut()
    }
}

/// Sometimes, Rust can't see through the lifetime. You can use this function to safely convince
/// Rust that `<T as By<'a, Val>>::Type` is `T`.
pub fn coerce_move<'a, T: By<'a, Val>>(by_val: T::Type) -> T {
    unsafe {
        let val = ::std::ptr::read(&by_val as *const <T as By<'a, Val>>::Type as *const T);
        ::std::mem::forget(by_val);
        val
    }
}

/// Sometimes, Rust can't see through the lifetime. You can use this function to safely convince
/// Rust that `<T as By<'a, Ref>>::Type` is `&'a T`.
pub fn coerce_ref<'a, T: By<'a, Ref>>(by_ref: T::Type) -> &'a T {
    unsafe { ::std::ptr::read(&by_ref as *const <T as By<'a, Ref>>::Type as *const &'a T) }
}

/// Sometimes, Rust can't see through the lifetime. You can use this function to safely convince
/// Rust that `<T as By<'a, Mut>>::Type` is `&'a mut T`.
pub fn coerce_mut<'a, T: By<'a, Mut>>(by_mut: T::Type) -> &'a mut T {
    unsafe { ::std::ptr::read(&by_mut as *const <T as By<'a, Mut>>::Type as *const &'a mut T) }
}

mod sealed {
    use super::*;

    pub trait Convention {}
    impl Convention for Val {}
    impl Convention for Ref {}
    impl Convention for Mut {}
}
