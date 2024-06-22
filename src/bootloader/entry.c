#include "print.h"
#include "file.h"
#include "debug.h"

void load_kernel(void)
{
    printf("Loading kernel ...\n");
    init_fs();
    if(!load_file("KERNEL.BIN", 0xFFFFFFFF80000000))
    {
        ASSERT(0, "Cannot find KERNEL.BIN");
    }
}
