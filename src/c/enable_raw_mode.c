#include <termios.h>
#include <unistd.h>
#include "enable_raw_mode.h"

struct termios  enable_raw_mode() {
    struct termios original;
    struct termios raw;
    tcgetattr(STDIN_FILENO, &original);
    raw = original;
    // Not との積をとるので echo のフラグは必ず0
    // に、それ以外は１をかけるのでそのままの状態になる.
    raw.c_lflag &= ~(ECHO);
    // TSCAFLUSH defines when to apply the change.
    // in this case, it waits for all pending output to be written to the terminal and discard any input that hasn't been read
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw);
    return original;
}

int  restore(struct termios * original) {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &original);
}
