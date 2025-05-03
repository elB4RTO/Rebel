use crate::panic::*;
use crate::memory::address::AddressError;
use crate::memory::{MemoryError, PagingError, TracingError, TracingPageError};


/// Utility functions to be implemented on [`Result`]
pub(crate)
trait OkOrPanic<E>
    where E: Panic
{
    /// Panics if the [`Result`] is [`Err`]
    fn ok_or_panic(&self);
}


impl OkOrPanic<MemoryError> for Result<(), MemoryError> {
    fn ok_or_panic(&self) {
        if let Err(e) = self {
            e.panic();
        }
    }
}


impl OkOrPanic<PagingError> for Result<(), PagingError> {
    fn ok_or_panic(&self) {
        if let Err(e) = self {
            e.panic();
        }
    }
}


impl OkOrPanic<TracingError> for Result<(), TracingError> {
    fn ok_or_panic(&self) {
        if let Err(e) = self {
            e.panic();
        }
    }
}


impl OkOrPanic<TracingPageError> for Result<(), TracingPageError> {
    fn ok_or_panic(&self) {
        if let Err(e) = self {
            e.panic();
        }
    }
}


impl OkOrPanic<AddressError> for Result<(), AddressError> {
    fn ok_or_panic(&self) {
        if let Err(e) = self {
            e.panic();
        }
    }
}
