(module
  (type (;0;) (func))
  (type (;1;) (func (param i32)))
  (type (;2;) (func (param i64)))
  (type (;3;) (func (param f64)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (import "env" "print_i32" (func (;0;) (type 1)))
  (import "env" "print_i64" (func (;1;) (type 2)))
  (import "env" "print_f64" (func (;2;) (type 3)))
  (memory (;0;) 1 10)
  (export "main" (func 5))
  (func (;3;) (type 0)
    local.get 0
    local.get 1
    i64.add
    drop
  )
  (func (;4;) (type 0)
    local.get 0
    local.get 1
    i64.mul
    drop
  )
  (func (;5;) (type 0)
    i64.const 42
    local.set 0
    i64.const 10
    local.set 1
    local.get 0
    local.get 1
    i64.add
    call 1
    i64.const 5
    i64.const 7
    call 3
    call 1
    i64.const 6
    i64.const 7
    call 4
    call 1
  )
)
