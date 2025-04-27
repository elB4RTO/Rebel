mod allocator;


use crate::memory::paging;


pub(crate) fn run() {
    paging::tests::run();

    allocator::run();
}
