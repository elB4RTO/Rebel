use crate::panic::*;
use crate::memory::address::AddressError;
use crate::memory::{MemoryError, PagingError, TracingError, TracingPageError};


/// Utility functions to be implemented on [`Result`]
pub(crate)
trait GetOrPanic<V,E>
    where E: Panic
{
    /// Retrieves the value contained in the [`Result`] if [`Ok`]
    /// or panics if [`Err`]
    fn get_or_panic(self) -> V;
}


impl<V> GetOrPanic<V, MemoryError> for Result<V, MemoryError> {
    fn get_or_panic(self) -> V {
        match self {
            Ok(v) => return v,
            Err(e) => e.panic(),
        }
    }
}


impl<V> GetOrPanic<V, PagingError> for Result<V, PagingError> {
    fn get_or_panic(self) -> V {
        match self {
            Ok(v) => return v,
            Err(e) => e.panic(),
        }
    }
}


impl<V> GetOrPanic<V, TracingError> for Result<V, TracingError> {
    fn get_or_panic(self) -> V {
        match self {
            Ok(v) => return v,
            Err(e) => e.panic(),
        }
    }
}


impl<V> GetOrPanic<V, TracingPageError> for Result<V, TracingPageError> {
    fn get_or_panic(self) -> V {
        match self {
            Ok(v) => return v,
            Err(e) => e.panic(),
        }
    }
}


impl<V> GetOrPanic<V, AddressError> for Result<V, AddressError> {
    fn get_or_panic(self) -> V {
        match self {
            Ok(v) => return v,
            Err(e) => e.panic(),
        }
    }
}
