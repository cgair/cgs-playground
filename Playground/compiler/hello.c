#include <stdio.h>

#define STR "Hello, world!\n"   // 变量式宏定义 (Object-like Macro)
#define MAX ((a)>(b)?(a):(b))   // 函数式宏定义 (Function-like Macro)

int main() {
    int k;
    int a[] = {0, 1};

    // k = MAX(a[0], a[1]);

    printf(STR);
    return 0;
}