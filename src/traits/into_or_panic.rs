use crate::panic::Panic;


pub(crate)
trait IntoOrPanic<V,E>
    where E: Panic,
{
    /// Converts the value and returns it or panics on failure
    fn into_or_panic(self) -> V;
}
