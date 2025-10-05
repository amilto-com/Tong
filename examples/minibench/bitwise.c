#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static long long now_ms(void){
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec*1000 + ts.tv_nsec/1000000;
}

int main(int argc, char** argv){
    long long n = (argc>1) ? atoll(argv[1]) : 2000000;
    long long t0 = now_ms();
    unsigned int x = 0;
    for(long long i=0;i<n;i++) x = ((x << 1) ^ (unsigned int)i) & 0x7fffffff;
    long long t1 = now_ms();
    printf("bitwise x=%u time_ms=%lld\n", x, (t1-t0));
    return 0;
}
