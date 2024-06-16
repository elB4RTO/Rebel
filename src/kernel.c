#include "bootloader/print.h"

void kernel_main(void)
{
    clear_screen();
    printf("Welcome in the Kernel\n");
    while (1) {}
}
