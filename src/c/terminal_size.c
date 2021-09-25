#include "terminal_size.h"

#include <stdio.h>
#include <unistd.h>
#include <sys/ioctl.h>

int terminal_size(struct TermSize *size) {
    struct winsize ws;
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0) {
        return -1;
    } else {
        size->col = ws.ws_col;
        size->row = ws.ws_row;
        return 0;
    }
}
