(module
 (type $none_=>_i64 (func (result i64)))
 (type $i64_=>_i64 (func (param i64) (result i64)))
 (type $f64_=>_none (func (param f64)))
 (type $none_=>_f64 (func (result f64)))
 (memory $0 0)
 (global $index/s0 (mut i64) (i64.const 0))
 (global $index/s1 (mut i64) (i64.const 0))
 (export "get_s0" (func $index/get_s0))
 (export "get_s1" (func $index/get_s1))
 (export "splitmix64" (func $index/splitmix64))
 (export "splitmix64_next_seed" (func $index/splitmix64_next_seed))
 (export "initState" (func $index/initState))
 (export "next" (func $index/next))
 (export "memory" (memory $0))
 (func $index/get_s0 (result i64)
  global.get $index/s0
 )
 (func $index/get_s1 (result i64)
  global.get $index/s1
 )
 (func $index/splitmix64 (param $0 i64) (result i64)
  local.get $0
  i64.const 7046029254386353131
  i64.sub
  local.tee $0
  local.get $0
  i64.const 30
  i64.shr_u
  i64.xor
  i64.const -4658895280553007687
  i64.mul
  local.tee $0
  local.get $0
  i64.const 27
  i64.shr_u
  i64.xor
  i64.const -7723592293110705685
  i64.mul
  local.tee $0
  local.get $0
  i64.const 31
  i64.shr_u
  i64.xor
 )
 (func $index/splitmix64_next_seed (param $0 i64) (result i64)
  local.get $0
  i64.const 7046029254386353131
  i64.sub
 )
 (func $index/initState (param $0 f64)
  (local $1 i64)
  (local $2 i64)
  local.get $0
  i64.reinterpret_f64
  local.tee $1
  i64.const 7046029254386353131
  i64.sub
  local.tee $2
  local.get $2
  i64.const 30
  i64.shr_u
  i64.xor
  i64.const -4658895280553007687
  i64.mul
  local.tee $2
  local.get $2
  i64.const 27
  i64.shr_u
  i64.xor
  i64.const -7723592293110705685
  i64.mul
  local.tee $2
  local.get $2
  i64.const 31
  i64.shr_u
  i64.xor
  global.set $index/s0
  local.get $1
  i64.const 4354685564936845354
  i64.add
  local.tee $1
  local.get $1
  i64.const 30
  i64.shr_u
  i64.xor
  i64.const -4658895280553007687
  i64.mul
  local.tee $1
  local.get $1
  i64.const 27
  i64.shr_u
  i64.xor
  i64.const -7723592293110705685
  i64.mul
  local.tee $1
  local.get $1
  i64.const 31
  i64.shr_u
  i64.xor
  global.set $index/s1
 )
 (func $index/next (result f64)
  (local $0 i64)
  (local $1 i64)
  global.get $index/s0
  local.tee $0
  global.get $index/s1
  i64.xor
  local.tee $1
  local.get $0
  i64.const 24
  i64.rotl
  i64.xor
  local.get $1
  i64.const 16
  i64.shl
  i64.xor
  global.set $index/s0
  local.get $1
  i64.const 37
  i64.rotl
  global.set $index/s1
  local.get $0
  i64.const 5
  i64.mul
  i64.const 7
  i64.rotl
  i64.const 9
  i64.mul
  i64.const 12
  i64.shr_u
  i64.const 4607182418800017408
  i64.or
  f64.reinterpret_i64
  f64.const 1
  f64.sub
 )
)
