#include "export_lib.h"

int main(void) {
    if (add(2, 3) != 5) return 1;
    Vec2 a = { 1.0, 2.0 };
    Vec2 b = { 3.0, 4.0 };
    double r = dot(a, b);
    if (r != 11.0) return 2;
    return 0;
}

