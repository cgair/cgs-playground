#include <stdio.h>
// #include <stdlib.h>
// #include <unistd.h>

int main(int argc, char *argv[], char *envp[]) {
    FILE * fp = fopen("a.txt", "w");
    fprintf(fp, "Hello, World");
}   // gdb 调试它 p *fp 看 _fileno
    // p *stdin (_fileno = 0)