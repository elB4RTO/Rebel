#ifndef KERNEL_ENTRY_LIB_H
#define KERNEL_ENTRY_LIB_H

#include <stdbool.h>

// these functions are defined in 'lib.asm'
// they're pretty much the same as the C standard library functions

// sets the memory. buffer is the address of the memory being set, value is the value we will assign to the memory, size is the size of the memory to be set
void memset(void* buffer, const char value, const int size);

// moves the memory to another location. dst is the destination memory address, src is the source memory address, size is the size of the memory to move
void memmove(void* dst, void* src, const int size);

// copies the memory to another location. dst is the destination memory address, src is the source memory address, size is the size of the memory to copy
void memcpy(void* dst, const void *const src, const int size);

// compares the values in two memory addresses. returns 0 if they're equal, non-0 otherwise. src1 and src2 are the memory locations to compare, size is the size of the memory to compare
int memcmp(const void *const src1, const void *const scr2, const int size);

#endif // KERNEL_ENTRY_LIB_H
