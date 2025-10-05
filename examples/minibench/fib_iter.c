#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static long long now_ms(void){
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec*1000 + ts.tv_nsec/1000000;
}

static long long fib_iter(long long n){
    if(n<=1) return n;
    long long a=0,b=1;
    for(long long i=2;i<=n;i++){ long long t=a+b; a=b; b=t; }
    return b;
}

int main(int argc, char** argv){
    long long n = (argc>1) ? atoll(argv[1]) : 40;
    long long t0 = now_ms();
    long long r = fib_iter(n);
    long long t1 = now_ms();
    printf("fib_iter(%lld)=%lld time_ms=%lld\n", n, r, (t1-t0));
    return 0;
}
