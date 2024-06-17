#ifndef KERNEL_ENTRY_DEBUG_H
#define KERNEL_ENTRY_DEBUG_H

#include "print.h"

#include <stdint.h>

#define HALT while(1){}

#define ERROR_MESSAGE(...) printf("Assertion failed [%s:%u]: %s", __VA_ARGS__);

#define ASSERT(TEST,MSG) do { if (!(TEST)) { ERROR_MESSAGE(__FILE__,__LINE__,MSG) HALT } } while(0);

#endif // KERNEL_ENTRY_DEBUG_H
