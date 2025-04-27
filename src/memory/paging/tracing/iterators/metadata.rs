use crate::memory::paging::tracing::{
    METADATA_ARRAY_SIZE, Metadata,
};


/// Iterates through all the entries of a metadata array
///
/// The iterator stops at the first entry of none type or when the end
/// of the array is reached.
pub(in crate::memory::paging::tracing)
struct MetadataIterator {
    metadata : *const Metadata,
    index    : usize,
}

impl MetadataIterator {
    /// Creates a new [`MetadataIterator`] upon the given [`Metadata`] array
    /// and starting from the given `index`
    pub(in crate::memory::paging::tracing)
    fn new(md_array:&[Metadata; METADATA_ARRAY_SIZE], idx:usize) -> Self {
        Self {
            metadata : &md_array[0] as *const Metadata,
            index    : idx,
        }
    }
}

impl Iterator for MetadataIterator {
    type Item = *const Metadata;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < METADATA_ARRAY_SIZE { unsafe {
            let item = self.metadata.add(self.index);
            self.index += 1;
            match (*item).is_none() {
                true => return None,
                _ => return Some(item),
            }
        }}
        None
    }
}


/// Iterates through all the entries of a metadata array
///
/// The iterator stops at the first entry of none type or when the end
/// of the array is reached.
pub(in crate::memory::paging::tracing)
struct MetadataIteratorMut {
    metadata : *mut Metadata,
    index    : usize,
}

impl MetadataIteratorMut {
    /// Creates a new [`MetadataIteratorMut`] upon the given [`Metadata`] array
    /// and starting from the given `index`
    pub(in crate::memory::paging::tracing)
    fn new(md_array:&mut [Metadata; METADATA_ARRAY_SIZE], idx:usize) -> Self {
        Self {
            metadata : &mut md_array[0] as *mut Metadata,
            index    : idx,
        }
    }
}

impl Iterator for MetadataIteratorMut {
    type Item = *mut Metadata;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < METADATA_ARRAY_SIZE { unsafe {
            let item = self.metadata.add(self.index);
            self.index += 1;
            match (*item).is_none() {
                true => return None,
                _ => return Some(item),
            }
        }}
        None
    }
}
