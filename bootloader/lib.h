#ifndef KERNEL_ENTRY_LIB_H
#define KERNEL_ENTRY_LIB_H

/// sets the memory location pointed to by `buffer` to the value provided by `value` for a size provided by `size`
void memset(void* buffer, const unsigned char value, const unsigned int size);

/// moves the content of the memory location pointed to by `src` into the location pointed to by `dst` for a size provided by `size`
void memmove(void *const dst, void *const src, const unsigned int size);

/// compares the content of the memory location pointed to by `ptr1` with the content of the memory location pointed to by `ptr2` for a size provided by `size`
/// returns 0 if equal, 1 otherwise
int memcmp(const void *const ptr1, const void *const ptr2, const unsigned int size);

#endif // KERNEL_ENTRY_LIB_H
