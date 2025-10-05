CC ?= gcc
CFLAGS ?= -O3 -march=native -pipe -std=c11
LDFLAGS ?=

PROGS = sum_loop sum_float fib_iter fib_rec array_fill array_sum bitwise call_overhead branch math_sin

all: $(PROGS)

%: %.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) -lm

clean:
	rm -f $(PROGS)
