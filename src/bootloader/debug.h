#ifndef KERNEL_ENTRY_DEBUG_H
#define KERNEL_ENTRY_DEBUG_H

#include "stdint.h"

// only using an if statement may cause problems, so wrap it inside a do-while loop
#define ASSERT(TEST,MSG) do { if (!(TEST)) error_message(__FILE__,__LINE__,MSG); while(1){} } while(0);

void error_message(const char* file, const uint64_t line, const char* msg);

#endif // KERNEL_ENTRY_DEBUG_H
