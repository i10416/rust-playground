#include <stdint.h>
struct TermSize  {
    unsigned short row;
    unsigned short col;
};

int terminal_size(struct TermSize * size);
