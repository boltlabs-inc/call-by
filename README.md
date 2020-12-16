To parameterize a function by calling convention, we can specify that it takes some `T:
CallBy<'a, Convention>`, and say that its input is of type `<T as CallBy<'a,
Convention>>::Type`. This is essentially a defunctionalization of Rust's reference operators.

This trick can be used to permit the *implementor* of a trait to pick the calling convention for
a value passed into (or out of) a function defined in that trait, rather than this being
hardcoded in the trait definition.

## Examples

For instance, say we wanted to define an abstraction for channels that can send values. Imagine,
however, that some channels might need to take ownership of the values they send, while others
might serialize values given only a reference to that value. In order to unify these two notions
into one trait, we can parameterize over the calling convention for the input value:

```rust
use call_by::{CallBy, CallingConvention};

trait Sender<'a, T>
where
    T: CallBy<'a, Self::Convention>,
{
    type Convention: CallingConvention;
    fn send(&self, value: <T as CallBy<'a, Self::Convention>>::Type);
}
```

Implementers of the `Sender` trait can choose whether the associated type `Convention` should be
`Val`, `Ref`, or `Mut`, which toggles the result of `<T as CallBy<'a, Self::Convention>>::Type`
between `T`, `&'a T`, and `&'a mut T`, respectively. Meanwhile, callers of the `send` method on
concretely known types don't need to specify the calling convention; the type-level function
determines what type they need to pass as the argument to `send`, and type errors are reported
in reference to that concrete type if it is known at the call site.
