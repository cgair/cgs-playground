#include <stdio.h>
#include "thread.h"

void use_printf(const char *s) {
  printf("%s", s);
}

void use_putchar(const char *s) {
  for (; *s; s++) {
    putchar(*s);
  }
}

void (*print)(const char *) = use_printf;

void Tworker() {
  char buf[128];
  int c = gettid() % 4 + 1;
  sprintf(buf, "\033[3%dm%d\033[0m", c, c);   // printf 的原子性?
  while (1) {
    print(buf);
  }
}

int main(int argc, char *argv[]) {
  if (argc > 1) {
    print = use_putchar;
    // print = use_printf;
  }

  setbuf(stdout, NULL);   // no buffer mode for standard output
  for (int i = 0; i < 4; i++) {
    create(Tworker);
  }
}
