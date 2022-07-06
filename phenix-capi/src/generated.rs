#[no_mangle]
pub extern "C" fn phenix_runtime_uint_encode(value: u64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&<phenix_runtime::Uint>::from(value), stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_uint_encode_many(
    values: *const u64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values.cast::<phenix_runtime::Uint>(), n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_sint_encode(value: i64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&<phenix_runtime::Sint>::from(value), stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_sint_encode_many(
    values: *const i64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values.cast::<phenix_runtime::Sint>(), n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_float_encode(value: f64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&<phenix_runtime::Float>::from(value), stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_float_encode_many(
    values: *const f64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values.cast::<phenix_runtime::Float>(), n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_bool_encode(value: bool, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_bool_encode_many(
    values: *const bool,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u8_encode(value: u8, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u8_encode_many(
    values: *const u8,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u16_encode(value: u16, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u16_encode_many(
    values: *const u16,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u32_encode(value: u32, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u32_encode_many(
    values: *const u32,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u64_encode(value: u64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_u64_encode_many(
    values: *const u64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i8_encode(value: i8, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i8_encode_many(
    values: *const i8,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i16_encode(value: i16, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i16_encode_many(
    values: *const i16,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i32_encode(value: i32, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i32_encode_many(
    values: *const i32,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i64_encode(value: i64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_i64_encode_many(
    values: *const i64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_f32_encode(value: f32, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_f32_encode_many(
    values: *const f32,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_f64_encode(value: f64, stream: *mut libc::FILE) -> libc::c_int {
    crate::call_encode(&value, stream)
}

#[no_mangle]
pub extern "C" fn phenix_runtime_f64_encode_many(
    values: *const f64,
    n: usize,
    stream: *mut libc::FILE,
) -> libc::c_int {
    crate::call_encode_many(values, n, stream)
}
