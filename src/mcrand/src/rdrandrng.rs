use super::RngCore;
use rand_core::{Error, impls};

// A implementation of RngCore which wraps calls to RDRAND instruction
// Should work in enclave and out of enclave with no changes
pub struct RdRandRng;

impl RdRandRng {
    pub fn new() -> RdRandRng {
        RdRandRng
    }
}

// See docu e.g.: https://docs.rs/rand_core/0.3.0/rand_core/trait.RngCore.html
impl RngCore for RdRandRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdrand32_step;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdrand32_step;

        let mut val: u32 = 0;
        let errcode = unsafe { _rdrand32_step(&mut val) };
        if 1 != errcode {
            panic!("_rdrand32_step unexpectedly failed!");
        }
        val
    }
    // TODO(chbeck) GH issue #578: Could implement next_u64 on x86_64 arch
    //       using _rdrand64_step
    //       I imagine that might make fill_bytes 2x faster
    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_u32(self)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}
