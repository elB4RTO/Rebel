#include "debug.h"
#include "print.h"

void error_message(const char* file, const uint64_t line, const char* msg)
{
    printf("Assertion failed [%s:%u]: %s", file, line, msg);
}
