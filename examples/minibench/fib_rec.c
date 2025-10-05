#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static long long now_ms(void){
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec*1000 + ts.tv_nsec/1000000;
}

static long long fib_rec(long long n){
    if(n<=1) return n;
    return fib_rec(n-1) + fib_rec(n-2);
}

int main(int argc, char** argv){
    long long n = (argc>1) ? atoll(argv[1]) : 28;
    long long t0 = now_ms();
    long long r = fib_rec(n);
    long long t1 = now_ms();
    printf("fib_rec(%lld)=%lld time_ms=%lld\n", n, r, (t1-t0));
    return 0;
}
