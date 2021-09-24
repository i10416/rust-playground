#include <termio.h>

struct termios enable_raw_mode();
int restore(struct termios * original);
