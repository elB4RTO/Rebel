#include "print.h"
#include "lib.h"

#include <stdint.h>
#include <stdarg.h>

// 80 columns, 2 Bytes each: 1 for the character and 1 for the descriptor
#define LINE_SIZE 160
#define COLUMNS 80
#define ROWS 25

#define FG_WHITE 0xF

struct ScreenBuffer
{
    char* buffer;
    int col;
    int row;
};

static struct ScreenBuffer screen_buffer = {(char*)(0xB8000), 0, 1}; // start from row 1 to not overwrite "Reading disk ..."

// returns the number of characters added to the buffer
static int udecimal_to_string(char* buffer, int position, uint64_t digits)
{
    const char digits_map[10] = "0123456789"; // no need to have the null terminating character here, this is only used to map between UINT and CHAR

    char digits_buffer[32];
    int size = 0;

    do
    {
        digits_buffer[size++] = digits_map[digits % 10];
        digits /= 10;
    }
    while (digits != 0);

    for (int i = size-1; i >= 0; --i) // characters in digits_buffer are in reverse order
    {
        buffer[position++] = digits_buffer[i];
    }

    return size;
}

// returns the number of characters added to the buffer
static int decimal_to_string(char* buffer, int position, int64_t digits)
{
    int size = 0;

    if (digits < 0)
    {
        digits = -digits; // given the use case, this should not overflow
        buffer[position++] = '-';
        size = 1;
    }

    size += udecimal_to_string(buffer, position, (uint64_t)digits);

    return size;
}

// returns the number of characters added to the buffer
static int hex_to_string(char* buffer, int position, uint64_t digits)
{
    const char digits_map[16] = "0123456789ABCDEF"; // no need to have the null terminating character here, this is only used to map between HEX and CHAR

    char digits_buffer[32];
    int size = 0;

    do
    {
        digits_buffer[size++] = digits_map[digits % 16];
        digits /= 16;
    }
    while (digits != 0);

    buffer[position++] = 'x';
    for (int i = size-1; i >= 0; --i) // characters in digits_buffer are in reverse order
    {
        buffer[position++] = digits_buffer[i];
    }

    return size+1;
}

// returns the number of characters added to the buffer
static int read_string(char *buffer, int position, const char* string)
{
    int index = 0;

    for (; string[index] != '\0'; ++index)
    {
        buffer[position++] = string[index];
    }

    return index;
}

static void write_screen(const char* buffer, int size, char color)
{
    struct ScreenBuffer* sb = &screen_buffer;
    int col = sb->col;
    int row = sb->row;

    for (int i = 0; i < size; ++i)
    {
        if (buffer[i] == '\n')
        {
            col = 0;
            ++row;
        }
        else if (buffer[i] == '\b')
        {
            if (col == 0 && row == 0)
            {
                continue;
            }
            else if (col == 0)
            {
                col = COLUMNS;
                --row;
            }
            --col;
            sb->buffer[col*2 + row*LINE_SIZE] = 0;
            sb->buffer[col*2 + row*LINE_SIZE + 1] = 0;
        }
        else
        {
            sb->buffer[col*2 + row*LINE_SIZE] = buffer[i];
            sb->buffer[col*2 + row*LINE_SIZE + 1] = color;

            if (++col >= COLUMNS)
            {
                col = 0;
                ++row;
            }
        }

        if (row >= ROWS)
        {
            // simulates a screen scroll, by copying every line data on the upper line, leaving the last line empty and free to use
            memmove(sb->buffer, sb->buffer+LINE_SIZE, LINE_SIZE*24);
            memset(sb->buffer+LINE_SIZE*24, 0, LINE_SIZE);
            --row;
        }

    }

    sb->col = col;
    sb->row = row;
}

int printf(const char *format, ...)
{
    char buffer[1024];
    int buffer_size = 0;
    int64_t integer = 0;
    char *string = 0;
    va_list args;

    va_start(args,format);

    for (int i = 0; format[i] != '\0'; i++)
    {
        if (format[i] != '%')
        {
            buffer[buffer_size++] = format[i];
        }
        else
        {
            switch (format[++i])
            {
            case 'x': // hexadecimal
                integer = va_arg(args, int64_t);
                buffer_size += hex_to_string(buffer, buffer_size, (uint64_t)integer);
                break;

            case 'u': // unsigned integer
                integer = va_arg(args, int64_t);
                buffer_size += udecimal_to_string(buffer, buffer_size, (uint64_t)integer);
                break;

            case 'i': case 'd': // signed integer
                integer = va_arg(args, int64_t);
                buffer_size += decimal_to_string(buffer, buffer_size, integer);
                break;

            case 's': // string
                string = va_arg(args, char*);
                buffer_size += read_string(buffer, buffer_size, string);
                break;

            default: // no need to support other formats
                buffer[buffer_size++] = '%';
                i--;
            }
        }
    }

    write_screen(buffer, buffer_size, FG_WHITE);

    va_end(args);

    return buffer_size;
}

void clear_screen()
{
    struct ScreenBuffer* sb = &screen_buffer;

    for (int row = 0; row < ROWS; ++row)
    {
        for (int col = 0; col < COLUMNS; ++col)
        {
            sb->buffer[col*2 + row*LINE_SIZE] = ' ';
            sb->buffer[col*2 + row*LINE_SIZE + 1] = 0;
        }
    }

    sb->col = 0;
    sb->row = 0;
}
