#![no_std]

extern crate rand_core;

/***
 * On x86 and x86_64 architectures, we don't depend on rand, and use rdrandrng.rs
 * Otherwise, we do depend on rand, and use fallback.rs
 */
#[cfg(not(target_arch = "x86_64"))]
extern crate rand;

pub use rand_core::RngCore;

#[cfg_attr(not(target_arch = "x86_64"), path = "fallback.rs")]
mod rdrandrng;
pub use rdrandrng::RdRandRng;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_entropy() {
        let first_result = RdRandRng {}.next_u32();
        for _ in 0..50 {
            let result = RdRandRng {}.next_u32();
            if result != first_result {
                return;
            }
        }
        panic!("Got the same u32 50 times in a row: {}", first_result);
    }
}
