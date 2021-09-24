#include <termio.h>

int enable_raw_mode(struct termios * original);
int restore(struct termios * original);
