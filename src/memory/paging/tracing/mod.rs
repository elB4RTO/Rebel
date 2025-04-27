mod iterators;
mod management;
mod page;
mod setup;

#[cfg(feature="unit_tests")]
pub(crate) mod tests;

use crate::panic::*;


pub(in crate::memory::paging) use iterators::*;
pub(in crate::memory::paging) use management::*;
pub(in crate::memory::paging) use page::*;
pub(crate) use setup::init_kernel_tracing_pages;


pub(crate)
enum TracingError {
    CleanupPreconditions,
    NotFound,
    InternalFailure,
    TracingPageError(TracingPageError),
    InvalidRequest,
}

impl From<TracingPageError> for TracingError {
    fn from(e:TracingPageError) -> Self {
        Self::TracingPageError(e)
    }
}

impl Panic for TracingError {
    fn panic(&self) -> ! {
        use TracingError::*;
        match self {
            CleanupPreconditions => panic("TracingError: CleanupPreconditions"),
            NotFound             => panic("TracingError: NotFound"),
            InternalFailure      => panic("TracingError: InternalFailure"),
            TracingPageError(_)  => panic("TracingError: TracingPageError"),
            InvalidRequest       => panic("TracingError: InvalidRequest"),
        }
    }
}


pub(crate)
enum TracingPageError {
    EntryIsNone,
    EntryIsFree,
    EntryIsTaken,
    EntrySizeMismatch,
    LeftShiftPreconditions,
    RightShiftPreconditions,
    SplitPreconditions,
    MergePreconditions,
    PushPreconditions,
    AppendPreconditions,
    PopPreconditions,
    ExtractPreconditions,
    RemovePreconditions,
    DropPreconditions,
    ResizePreconditions,
    InvalidRequest,
    InternalFailure,
    NotFound,
}

impl Panic for TracingPageError {
    fn panic(&self) -> ! {
        use TracingPageError::*;
        match self {
            EntryIsNone             => panic("TracingPageError: EntryIsNone"),
            EntryIsFree             => panic("TracingPageError: EntryIsFree"),
            EntryIsTaken            => panic("TracingPageError: EntryIsTaken"),
            EntrySizeMismatch       => panic("TracingPageError: EntrySizeMismatch"),
            LeftShiftPreconditions  => panic("TracingPageError: LeftShiftPreconditions"),
            RightShiftPreconditions => panic("TracingPageError: RightShiftPreconditions"),
            SplitPreconditions      => panic("TracingPageError: SplitPreconditions"),
            MergePreconditions      => panic("TracingPageError: MergePreconditions"),
            PushPreconditions       => panic("TracingPageError: PushPreconditions"),
            AppendPreconditions     => panic("TracingPageError: AppendPreconditions"),
            PopPreconditions        => panic("TracingPageError: PopPreconditions"),
            ExtractPreconditions    => panic("TracingPageError: ExtractPreconditions"),
            RemovePreconditions     => panic("TracingPageError: RemovePreconditions"),
            DropPreconditions       => panic("TracingPageError: DropPreconditions"),
            ResizePreconditions     => panic("TracingPageError: ResizePreconditions"),
            InvalidRequest          => panic("TracingPageError: InvalidRequest"),
            InternalFailure         => panic("TracingPageError: InternalFailure"),
            NotFound                => panic("TracingPageError: NotFound"),
        }
    }
}
