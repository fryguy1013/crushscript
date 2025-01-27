#include <stdint.h>
#include <time.h>

int32_t Number(char* str) {
    return atoi(str);
}

namespace Math {
    int32_t floor(double x) {
        return ::floor(x);
    }

    double random() {
        double r = ((double)rand() / (double)(RAND_MAX));
        return r;
    }
}

namespace console {
    void log(int32_t x) {
        printf("%d\n", x);
    }

    void log(char* x) {
        printf("%s\n", x);
    }
}

int32_t* js_constructor_Int32Array(int32_t size) {
    return new int[size];
}

namespace process {
    static char **argv;
    static int argc;

    static void setargs(int argc, char** argv) {
        process::argc = argc + 1;
        process::argv = new char*[argc + 1];

        // process::argv[0] = new char[strlen("node") + 1];
        // strcpy(process::argv[0], "node");
        for (int i=0; i<argc; i++)
            process::argv[i+1] = argv[i];

        srand(time(NULL));
    }
}