
arameterize a function by calling convention, we can specify that it takes some `T:
a, Convention>`, and say that its input is of type `<T as By<'a,
ention>>::Type`. This is essentially a defunctionalization of Rust's reference operators.

 trick can be used to permit the *implementor* of a trait to pick the calling convention for
lue passed into (or out of) a function defined in that trait, rather than this being
coded in the trait definition.

xamples

instance, say we wanted to define an abstraction for channels that can send values. Imagine,
ver, that some channels might need to take ownership of the values they send, while others
t serialize values given only a reference to that value. In order to unify these two notions
 one trait, we can parameterize over the calling convention for the input value:

ust
call_by::{By, Convention};

t Sender<'a, T>
e
T: By<'a, Self::Convention>,

type Convention: Convention;
fn send(&self, value: <T as By<'a, Self::Convention>>::Type);



ementers of the `Sender` trait can choose whether the associated type `Convention` should be
`, `Ref`, or `Mut`, which toggles the result of `<T as By<'a, Self::Convention>>::Type`
een `T`, `&'a T`, and `&'a mut T`, respectively. Meanwhile, callers of the `send` method on
retely known types don't need to specify the calling convention; the type-level function
rmines what type they need to pass as the argument to `send`, and type errors are reported
eference to that concrete type if it is known at the call site.
 snip -->


There are three fundamental ways to pass a `T` as input or return a `T` as output: by [`Val`]ue,
by shared immutable [`Ref`]erence, and by unique [`Mut`]able reference.

This is a sealed trait, implemented for all three of these conventions.
trait Convention: sealed::Convention + Sized {
const TOKEN: Self;


 Convention for Val {
const TOKEN: Self = Val;


 Convention for Ref {
const TOKEN: Self = Ref;


 Convention for Mut {
const TOKEN: Self = Mut;


To get the type of `T` via calling convention `Convention`, write `<T as By<'a,
Convention>>::Type`.
trait By<'a, C: Convention> {
/// The type of `Self` when called by `Convention`.
type Type;


Taking a `T` by [`Val`]ue means taking a `T` as input to or output from a function.
rive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Val;

<'a, T> By<'a, Val> for T {
type Type = T;


Taking a `T` by [`Ref`]erence means taking `&'a T` as input to or output from a function.
rive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Ref;

<'a, T: 'a> By<'a, Ref> for T {
type Type = &'a T;


Taking a `T` by [`Mut`]able reference means taking `&'a mut T` as input to or output from a
function.
rive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Mut;

<'a, T: 'a> By<'a, Mut> for T {
type Type = &'a mut T;


Convert between different calling conventions.

Only some conversions are sensible in Rust, due to the ownership system. These are the valid
conversions, with the constraints on the underlying type `T` noted:

| Can I convert... | ... to [`Val`] (`T`)  | ... to [`Ref`] (`&'a T`) | ... to [`Mut`] (`&'a mut T`) |
| :--------------------------------- | :------------------ | :------ | :------ |
| **from [`Val`] (`T`) ...**         | (valid for all `T`) | ❌*     | ❌*     |
| **from [`Ref`] (`&'a T`) ...**     | `T: 'a +` [`Clone`] | `T: 'a` | ❌**    |
| **from [`Mut`] (`&'a mut T`) ...** | `T: 'a +` [`Clone`] | `T: 'a` | `T: 'a` |

> \* Impossible because references can't outlive the data they borrow.
>
> \** Impossible because potentially-aliased data can't be mutably referenced.
trait Convert<'a, From: Convention, To: Convention>
e
Self: By<'a, To> + By<'a, From>,

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


<'a, T> Convert<'a, Val, Val> for T {
fn convert(from: T) -> T {
    from
}


<'a, T: 'a + Clone> Convert<'a, Ref, Val> for T {
fn convert(from: &T) -> T {
    from.clone()
}


<'a, T: 'a + Clone> Convert<'a, Mut, Val> for T {
fn convert(from: &mut T) -> T {
    from.clone()
}


<'a, T: 'a> Convert<'a, Ref, Ref> for T {
fn convert(from: &T) -> &T {
    from
}


<'a, T: 'a> Convert<'a, Mut, Ref> for T {
fn convert(from: &mut T) -> &T {
    &*from
}


<'a, T: 'a> Convert<'a, Mut, Mut> for T {
fn convert(from: &mut T) -> &mut T {
    from
}


The generalization of [`Into`], [`AsRef`], and [`AsMut`]: in a calling-convention polymorphic
context, this trait allows you to invoke the appropriate conversion method depending on the
applicable calling convention.

# Examples

```
use call_by::*;

fn do_something<'a, T, S, C>(input: <S as By<'a, C>>::Type)
where
    T: By<'a, C>,
    S: By<'a, C> + As<'a, C, T>,
    C: Convention,
{
    let t: <T as By<'a, C>>::Type = S::as_convention(input);
    // ... do something with `t` ...
}
```
trait As<'a, C: Convention, T: By<'a, C>>: By<'a, C> {
#[allow(clippy::wrong_self_convention)]
fn as_convention(this: <Self as By<'a, C>>::Type) -> <T as By<'a, C>>::Type;


<'a, T, S> As<'a, Val, T> for S
e
S: Into<T>,

fn as_convention(this: S) -> T {
    this.into()
}


<'a, T: 'a, S: 'a> As<'a, Ref, T> for S
e
S: AsRef<T>,

fn as_convention(this: &S) -> &T {
    this.as_ref()
}


<'a, T: 'a, S: 'a> As<'a, Mut, T> for S
e
S: AsMut<T>,

fn as_convention(this: &mut S) -> &mut T {
    this.as_mut()
}


sealed {
use super::*;

pub trait Convention {}
impl Convention for Val {}
impl Convention for Ref {}
impl Convention for Mut {}

