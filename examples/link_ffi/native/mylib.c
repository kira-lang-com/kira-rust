#include "mylib.h"
#include <stdlib.h>

struct Foo {
    int64_t value;
};

int64_t add64(int64_t a, int64_t b) { return a + b; }

FooRef foo_newa(void) {
    struct Foo* foo = (struct Foo*)malloc(sizeof(struct Foo));
    foo->value = 124;
    return foo;
}

int64_t foo_value(FooRef foo) { return foo->value; }

void foo_free(FooRef foo) { free(foo); }

