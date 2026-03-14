#pragma once
#include <stdint.h>

typedef struct Foo* FooRef;

int64_t add64(int64_t a, int64_t b);
FooRef foo_new(void);
int64_t foo_value(FooRef foo);
void foo_free(FooRef foo);

