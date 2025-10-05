#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>

static long long now_ms(void){
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec*1000 + ts.tv_nsec/1000000;
}

int main(int argc, char** argv){
    long long n = (argc>1) ? atoll(argv[1]) : 200000;
    long long t0 = now_ms();
    double s = 0.0;
    for(long long i=0;i<n;i++) s += sin((double)i);
    long long t1 = now_ms();
    printf("math_sin s=%.12f time_ms=%lld\n", s, (t1-t0));
    return 0;
}
