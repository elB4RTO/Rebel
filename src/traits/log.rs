use crate::memory::address::AddressError;
use crate::memory::{MemoryError, PagingError};


pub(crate)
trait Log {
    fn log(&self);

    fn log_and_then<F>(&self, f:F)
        where F: FnOnce()
    {
        self.log();
        f()
    }
}


impl Log for MemoryError {
    fn log(&self) {
        // TODO:
        // crate::tty::print("\n-------- MEMORY ERROR --------");
        // crate::tty::print("\nTODO: implement logging");
        // crate::tty::print("\n------------------------------");
    }
}


impl Log for PagingError {
    fn log(&self) {
        // TODO:
        // crate::tty::print("\n-------- PAGING ERROR --------");
        // crate::tty::print("\nTODO: implement logging");
        // crate::tty::print("\n------------------------------");
    }
}


impl Log for AddressError {
    fn log(&self) {
        // TODO:
        // crate::tty::print("\n-------- ADDRESS ERROR --------");
        // crate::tty::print("\nTODO: implement logging");
        // crate::tty::print("\n-------------------------------");
    }
}



pub(crate)
trait LogError<V,E>
    where E: Log
{
    /// Logs the contained error
    fn log_err(self) -> Result<V,E>;
    /// Logs the contained error and then replaces it
    fn log_map_err<F>(self, f:F) -> Result<V,F>;
}


impl<V> LogError<V, MemoryError> for Result<V, MemoryError> {
    fn log_err(self) -> Result<V,MemoryError> {
        self.map_err(|e|{ e.log(); e })
    }

    fn log_map_err<F>(self, f:F) -> Result<V,F> {
        self.map_err(|e|{ e.log(); f })
    }
}


impl<V> LogError<V, PagingError> for Result<V, PagingError> {
    fn log_err(self) -> Result<V,PagingError> {
        self.map_err(|e|{ e.log(); e })
    }

    fn log_map_err<F>(self, f:F) -> Result<V,F> {
        self.map_err(|e|{ e.log(); f })
    }
}


impl<V> LogError<V, AddressError> for Result<V, AddressError> {
    fn log_err(self) -> Result<V,AddressError> {
        self.map_err(|e|{ e.log(); e })
    }

    fn log_map_err<F>(self, f:F) -> Result<V,F> {
        self.map_err(|e|{ e.log(); f })
    }
}
