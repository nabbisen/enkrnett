# M-2 — Daemon footprint on `mipsel_24kc`

| | |
|---|---|
| **Measurement** | M-2 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-04 |
| **State** | Ready for review (sizes complete; run-time numbers on MIPS pending an emulator) |

## Question

How large is a daemon of Enkrnett's kind on `mipsel_24kc`, and how much memory does it use?

## Answer in one paragraph

Built with nightly Rust (`-Zbuild-std`) and the OpenWrt 25.12.5 SDK toolchain, optimised for size,
stripped, and dynamically linked against OpenWrt's musl as OpenWrt packages are: a std-only TCP
listener (program A) is **397 KB**; a blocking HTTP server with JSON, an argon2 password hash, OS
randomness, and a `ubus` call (program B) is **725 KB**, or **659 KB** when `ubus` is reached
through a small in-house socket client instead of spawning the `ubus` command; the same function
on axum + tokio (program C) is **1,053 KB**. Compressed as OpenWrt's squashfs compresses
(xz, preset 9e, 256 KiB blocks), these are about **128 KB, 234 KB (221 KB), and 341 KB**. Program B
therefore lands under the provisional budget of 350 KB in squashfs, program C does not. Static
linking adds about 65 KB raw (B: 790 KB, 266 KB compressed) and needs two workarounds in the link
step. Idle memory could so far be measured only for the x86-64 glibc builds (RSS 2.4–2.9 MB,
proportional share 0.3–0.8 MB); MIPS run-time numbers need `qemu-user` or `qemu-system-mips`,
which are not installed on the workstation.

## Environment

| Item | Version |
|---|---|
| Rust | `rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)`, toolchain `nightly-2026-08-29`, with `rust-src`; `-Zbuild-std=std,panic_abort` |
| Target | `mipsel-unknown-linux-musl` (Tier 3), dynamic (`-C target-feature=-crt-static`) |
| Linker and C toolchain | `openwrt-sdk-25.12.5-ramips-mt7621_gcc-14.3.0_musl`: `mipsel-openwrt-linux-musl-gcc (OpenWrt GCC 14.3.0 r33051-f5dae5ece4)` |
| Cargo profile | `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` |
| Host comparison | same programs, `x86_64-unknown-linux-gnu`, same profile |
| Workstation | 32 cores, 59 GB RAM, Linux 7.2.8 |

### The three programs

| Program | Does | Dependencies (crates, excluding std) |
|---|---|---|
| **A floor** | TCP listener; one fixed HTTP response | 0 |
| **B lean** | Blocking HTTP/1.1 with a hand-written parser; `GET/PUT /state` as JSON (serde, serde_json); `POST /setpw`, `POST /login` with argon2id (m = 8 MiB, t = 3) and, for the timing, pbkdf2-sha256; `GET /random` from `/dev/urandom`; `GET /ubus` = `ubus call system board` through `std::process::Command`, or (feature `native-ubus`) a 120-line client that speaks ubus's wire format on the unix socket and performs a `lookup` | 35: argon2, pbkdf2, sha2, serde, serde_json, rand_core (getrandom) and their transitive crates |
| **C asyncd** | The function of B on axum 0.8 + tokio 1.53 (current-thread runtime, `http1`, `json`) | 67 |

Source: `.git-exclude/tmp/m/m2/ws/` (outside the repository, as the handoff requires).

## Results

### Step 2–3: size

`raw` = stripped executable. `xz` = `xz --format=raw --lzma2=preset=9e,lc=0,lp=2,pb=2,dict=256KiB`,
the compressor and settings OpenWrt uses for squashfs; it approximates the cost inside the image
(squashfs compresses in independent 256 KiB blocks, so the true value is slightly larger).
`mksquashfs` is not installed on the workstation; the exact figure can be added later.

| Variant | Program | raw bytes | xz bytes |
|---|---|---:|---:|
| MIPS, dynamic musl (as OpenWrt packages) | A floor | 397,108 | 128,223 |
| | B lean | 724,988 | 234,184 |
| | B lean, in-house ubus client | 659,320 | 220,681 |
| | C asyncd | 1,052,808 | 340,577 |
| x86-64 glibc, for comparison | A floor | 297,776 | 129,706 |
| | B lean | 540,352 | 229,317 |
| | C asyncd | 838,608 | 341,599 |
| MIPS, static musl | A floor | 461,836 | 154,401 |
| | B lean | 789,756 | 265,500 |
| | C asyncd | 1,183,116 | 379,358 |

Static linking needed two workarounds: `-C link-self-contained=no`, because `rustc` otherwise
expects the self-contained `crt1.o`, `crti.o`, `crtbegin.o` that only a distributed Tier 2 target
ships; and a `libunwind.a` that points at the toolchain's `libgcc_eh.a`, because `std` links
`-lunwind` and `build-std` does not build one. With both, the three programs link and are
plain (non-PIE) static executables.

### Step 4–5: memory

MIPS run-time measurements are **pending**: the workstation has neither `qemu-user` (`qemu-mipsel`)
nor `qemu-system-mips`, and the measurement VM is x86-64. The host builds were measured instead,
as an indication only:

| Program (x86-64 glibc, dynamic) | Idle, 2 s | Idle, 62 s | After 1,000 requests + 20 logins |
|---|---|---|---|
| A floor | RSS 2,372 KiB, PSS 333 KiB | same | — |
| B lean | RSS 2,432 KiB, PSS 397 KiB | same | RSS 10,860 KiB, PSS 8,825 KiB |
| C asyncd | RSS 2,924 KiB, PSS 838 KiB | same | RSS 11,248 KiB, PSS 9,162 KiB |

The growth after load is the argon2 working memory (8 MiB) that the allocator keeps; it is
expected and bounded by the hash parameters.

### Step 6: password check time

Measured on the x86-64 host (3 verifications each). MIPS figures pending the emulator; a
580–880 MHz MIPS core without an FPU is expected to be one to two orders of magnitude slower.

| Function | Per verify, x86-64 |
|---|---:|
| argon2id m = 4 MiB, t = 4 | 5 ms |
| argon2id m = 8 MiB, t = 3 | 8 ms |
| argon2id m = 19 MiB, t = 2 | 12 ms |
| pbkdf2-sha256, 100,000 iterations | 22 ms |
| pbkdf2-sha256, 310,000 iterations | 69 ms |
| pbkdf2-sha256, 600,000 iterations | 134 ms |

### Step 7: two ways to reach ubus

| Way | Effort | Size effect (program B, MIPS) |
|---|---|---|
| `std::process::Command` running `ubus call` | Trivial | 724,988 B |
| In-house client on the unix socket (`lookup` only) | About 120 lines; the wire format is 8-byte header + nested length-prefixed attributes | 659,320 B (**66 KB smaller**: `Command` pulls in process-spawning code) |
| Dynamic linking to `libubus` | Not tried: the SDK ships no `libubus` headers or library in `staging_dir`; they would have to be built with the SDK first | — |

## Measured values

| Quantity | Value | Unit | Provisional budget |
|---|---:|---|---:|
| Program B inside squashfs (xz estimate) | 234 (221 with in-house ubus) | KB | ≤ 350 |
| Program C inside squashfs (xz estimate) | 341 | KB | ≤ 350 |
| Program A inside squashfs (xz estimate) | 128 | KB | — |
| Program B idle memory, x86-64 glibc | RSS 2.4 / PSS 0.4 | MB | ≤ 3 (on MIPS) |
| Password check, argon2id 8 MiB t 3, x86-64 | 8 | ms | ≤ 500 (on MIPS) |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| Run target: x86-64 host instead of `malta/le` or a device | No MIPS emulator installed; no device at hand. To complete: `qemu-user` (`qemu-mipsel` with the SDK's musl loader) or `qemu-system-mips` with the `malta/le` 25.12.5 image. |
| squashfs size approximated with xz | `squashfs-tools` not installed. |
| Static variant needed link workarounds | See the size table. OpenWrt packages are dynamic; the static figures are for reference. |
| Toolchain: nightly `build-std` instead of OpenWrt's packages-feed Rust | The feed builds rustc and LLVM from source (hours, tens of GB). Both routes produce dynamic musl binaries against the same libc; sizes should be close, not identical. |

## What contradicts an RFC

None. The numbers fall on both sides of the provisional budget (D-10) as the RFC anticipated:
a blocking daemon fits, an asynchronous web framework does not.

## Observations beyond the question

1. `build-std` with the SDK toolchain works out of the box for dynamic binaries: `rustup target add` is not needed, no patches, a one-line `.cargo` linker setting plus `STAGING_DIR`. The whole workspace (three programs, std included) builds in about 2 minutes on this workstation.
2. The dynamic MIPS binaries are PIE and expect `/lib/ld-musl-mipsel-sf.so.1`, which the OpenWrt image provides.
3. `std::process::Command` is expensive on this target (≈ 66 KB compressed-raw difference). A daemon that never spawns processes saves that.
4. The x86-64 and MIPS compressed sizes are nearly equal although the MIPS raw sizes are 30 % larger: MIPS code compresses better.
5. The dependency list of program B includes `proc-macro2`, `quote`, `syn` only at build time (serde derive); they do not reach the binary.

## How to repeat

```sh
cd .git-exclude/tmp/m/m2
./build.sh          # builds every variant; logs in out/<variant>.log
./build.sh sizes    # size table (adds the exact squashfs figure when mksquashfs is installed)
./rss.sh            # host memory and timing
```
