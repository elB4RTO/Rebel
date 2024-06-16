#include "print.h"
#include "file.h"
#include "debug.h"

void entry_main(void)
{
    printf("Loading kernel ...\n");
    init_fs();
    if(!load_file("KERNEL.BIN", 0x7400000))
    {
        ASSERT(0, "Cannot find KERNEL.BIN");
    }
}
