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
  (export "main" (func 8))
  (func (;3;) (type 0)
    local.get 0
    local.get 0
    i64.mul
    drop
  )
  (func (;4;) (type 0)
    local.get 0
    local.get 0
    i64.mul
    local.get 0
    i64.mul
    drop
  )
  (func (;5;) (type 0)
    local.get 0
    call 3
    local.get 1
    call 3
    i64.add
    drop
  )
  (func (;6;) (type 0)
    local.get 0
    local.get 1
    call 7
    drop
  )
  (func (;7;) (type 0)
    local.get 0
    local.get 0
    i64.mul
    drop
  )
  (func (;8;) (type 0)
    i64.const 5
    call 3
    call 1
    i64.const 3
    call 4
    call 1
    i64.const 3
    i64.const 4
    call 5
    call 1
    i64.const 8
    call 6
    call 1
    i64.const 12
    local.set 0
    i64.const 5
    local.set 1
    local.get 0
    local.get 1
    i64.and
    call 1
    local.get 0
    local.get 1
    i64.or
    call 1
    local.get 0
    local.get 1
    i64.xor
    call 1
    local.get 0
    i64.const 2
    i64.shl
    call 1
    local.get 1
    i64.const 1
    i64.shr_s
    call 1
    i64.const 10
    local.set 2
    i64.const 20
    local.set 3
    i64.const 30
    local.set 4
    local.get 2
    local.get 3
    i64.lt_s
    local.get 3
    local.get 4
    i64.lt_s
    i32.and
    call 1
    local.get 2
    local.get 3
    i64.gt_s
    local.get 3
    local.get 4
    i64.lt_s
    i32.or
    call 1
    local.get 2
    local.get 3
    i64.eq
    local.get 3
    local.get 4
    i64.eq
    i32.or
    call 1
  )
)
