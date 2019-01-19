mcrand
======

The goal of this crate is to provide a Rust API which

- Matches that of the standard library random generator objects
- Works both in the enclave and outside of enclave by calling rdrand instruction
- Does not require a cargo `sgx` feature to work, which simplifies the build

This is basically the same functionality as sgx_rand crate from rust_sgx_sdk.

However, instead of calling into the linux intel sgx driver at `sgx_read_rand`
function, we simply call the rust `rdrand` intrinsic, which calls the RDRAND
ops on x86.

This is provided even without the standard library, by libcore.
(c.f. https://github.com/rust-lang-nursery/stdsimd) 
So, it is expected to work both in the enclave and outside of it, even when
compiling with nostd.
