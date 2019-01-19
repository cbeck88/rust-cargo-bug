Bad linking of `std`
====================

This repository shows cargo wrongly linking rust `std` library to support a
conditional dependency that is not selected, which breaks the build because it
conflicts with the lang-items that we provided.

```
$ cargo --version
cargo 1.30.0
$ rustc --version
rustc 1.30.0
```

To reproduce the issue:

1. Run `cargo build`, or if you are not on `x86_64`, run `cargo build --target=x86_64-unknown-linux-gnu`

   Observe a build failure:

```
$ cargo build --target=x86_64-unknown-linux-gnu
   Compiling rand_core v0.3.0
   Compiling mcrand v1.0.0 (/home/chris/cargo-bug/src/mcrand)
   Compiling myenclave v1.0.0 (/home/chris/cargo-bug/src/myenclave)
error[E0152]: duplicate lang item found: `panic_impl`.
  --> src/myenclave/src/lib.rs:9:1
   |
9  | / fn panic(info: &PanicInfo) -> ! {
10 | |     loop {}
11 | | }
   | |_^
   |
   = note: first defined in crate `std`.

error: aborting due to previous error

For more information about this error, try `rustc --explain E0152`.

error: Could not compile `myenclave`.
```

2. Apply a patch that comments out rand dependency, which isn't even compiled
   as part of this build:

```
$ git apply 0001-Comment-out-conditional-dependency.patch
$ git diff
diff --git a/src/mcrand/Cargo.toml b/src/mcrand/Cargo.toml
index f570117..6e22e27 100644
--- a/src/mcrand/Cargo.toml
+++ b/src/mcrand/Cargo.toml
@@ -29,4 +29,4 @@ name = "mcrand"
 rand_core = { version = "0", default-features = false }

 [target.'cfg(not(target_arch = "x86_64"))'.dependencies]
-rand = { version = "0.6" }
+#rand = { version = "0.6" }
```

3. Observe that the build now passes:

```
$ cargo build
   Compiling rand_core v0.3.0
   Compiling mcrand v1.0.0 (/home/chris/cargo-bug/src/mcrand)
   Compiling myenclave v1.0.0 (/home/chris/cargo-bug/src/myenclave)
    Finished dev [unoptimized + debuginfo] target(s) in 0.40s
```

Yet, `rand` is not selected as a dependency, or even compiled as part of these
builds, so why does the presence or absence of `rand` change whether `std` is
linked?

Clearly, `std` should only be linked when `rand` is actually a dependency.

Distinguishing this issue from Cargo issue #2589 (https://github.com/rust-lang/cargo/issues/2589#issuecomment-224697180)
------------------------------------------------------------------------------------------------------------------------

It was written in 2016 by @alexcrichton:

> yeah unfortunately that's the same issue as this. When we build the resolution graph today it contains information about all targets, and the filtering per platform only happens at the very end when we're compiling.

However, that is a separate issue from the build failure that we see above.

We can see this by replacing the `cfg(target_arch = "x86")` expressions in our
example, with a feature `no_rdrand` on `mcrand` crate, which is not on by default,
and make the dependency on `rand` optional. In this case, the overall build passes,
and when the feature is on, `mcrand` detects the feature and depends on `rand`, and
builds on its own correctly with `rand` as well.

To see this:

1. Apply patch `0001-Use-features-instead-of-cfg-target_arch-.-expression.patch`:

```
$ git apply 0001-Use-features-instead-of-cfg-target_arch-.-expression.patch
$ git diff
diff --git a/src/mcrand/Cargo.toml b/src/mcrand/Cargo.toml
index f570117..b856a9d 100644
--- a/src/mcrand/Cargo.toml
+++ b/src/mcrand/Cargo.toml
@@ -27,6 +27,8 @@ name = "mcrand"

 [dependencies]
 rand_core = { version = "0", default-features = false }
+rand = { version = "0.6", optional = true }

-[target.'cfg(not(target_arch = "x86_64"))'.dependencies]
-rand = { version = "0.6" }
+[features]
+default = []
+no_rdrand = [ "rand" ]
diff --git a/src/mcrand/src/lib.rs b/src/mcrand/src/lib.rs
index 72f85e8..e0887bb 100644
--- a/src/mcrand/src/lib.rs
+++ b/src/mcrand/src/lib.rs
@@ -6,12 +6,12 @@ extern crate rand_core;
  * On x86 and x86_64 architectures, we don't depend on rand, and use rdrandrng.rs
  * Otherwise, we do depend on rand, and use fallback.rs
  */
-#[cfg(not(target_arch = "x86_64"))]
+#[cfg(feature = "no_rdrand")]
 extern crate rand;

 pub use rand_core::RngCore;

-#[cfg_attr(not(target_arch = "x86_64"), path = "fallback.rs")]
+#[cfg_attr(feature = "no_rdrand", path = "fallback.rs")]
 mod rdrandrng;
 pub use rdrandrng::RdRandRng;

```

2. Observe that the overall build now passes:

```
$ cargo build
   Compiling mcrand v1.0.0 (/home/chris/cargo-bug/src/mcrand)
   Compiling myenclave v1.0.0 (/home/chris/cargo-bug/src/myenclave)
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
```

3. Observe that building mcrand crate with `no_rdrand` feature enabled also passes
and selects rand as a dependency and builds it:

```
$ cd src/mcrand
$ cargo build --features no_rdrand
   Compiling semver-parser v0.7.0
   Compiling libc v0.2.47
   Compiling autocfg v0.1.2
   Compiling rand_core v0.3.0
   Compiling rand_xorshift v0.1.1
   Compiling rand_isaac v0.1.1
   Compiling rand_hc v0.1.0
   Compiling rand_chacha v0.1.1
   Compiling rand v0.6.4
   Compiling semver v0.9.0
   Compiling rustc_version v0.2.3
   Compiling rand_os v0.1.1
   Compiling rand_pcg v0.1.1
   Compiling mcrand v1.0.0 (/home/chris/cargo-bug/src/mcrand)
    Finished dev [unoptimized + debuginfo] target(s) in 3.90s
```
