#include "enable_raw_mode.h"

#include <termios.h>
#include <unistd.h>

int  enable_raw_mode(struct termios * original) {
    struct termios raw;

    if(tcgetattr(STDIN_FILENO, original)==-1){
        return -1;
    };

    raw = *original;
    // set flag so that break condition will cause a SIGINT
    // see https://www.cmrr.umn.edu/~strupp/serial.html#2_3_3
    // INPCK: parity check
    // ISTRIP: strip input
    // disable ctrl-S, ctrl-Q
    // read ctrl-M, Enter as 13
    raw.c_iflag &= ~(BRKINT | INPCK | ISTRIP | ICRNL | IXON);
    // read stdin byte by byte, disable echo, ctrl-C,ctrl-V, ctrl-Z
    raw.c_lflag &= ~(ECHO | ICANON | IEXTEN | ISIG);
    // set char size as 8bit per byte
    raw.c_cflag |= (CS8);

    // TSCAFLUSH defines when to apply the change.
    // do the same for output
    raw.c_oflag &= ~(OPOST);

    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

    // in this case, it waits for all pending output to be written to the
    // terminal and discard any input that hasn't been read
    if(tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw)==-1){
        return -1;
    };
    return 0;
}

int restore(struct termios* original) {
    return tcsetattr(STDIN_FILENO, TCSAFLUSH, &original);
}
