# M-2 — Daemon footprint on `mipsel_24kc`

| | |
|---|---|
| **Measurement** | M-2 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-04 |
| **State** | Ready for review |

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
step. On an emulated MIPS 24Kc running OpenWrt 25.12.5 (`malta/le`, 128 MB), the dynamic lean
daemon occupies **1.3–1.4 MB** resident (idle and after 300 requests plus password hashing),
the static one **0.7–0.8 MB**, the async framework **1.8 MB** dynamic; musl returns the argon2
working memory after each hash. Password checking with argon2id (8 MiB, t = 3) took 182 ms per
verify under emulation, which is not representative of real hardware.

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
The squashfs column is the exact figure.

`squashfs` = size of a squashfs image (xz, 256 KiB blocks, stock `mksquashfs` 4.x) that holds only
that file; it includes about 1 KiB of filesystem metadata.

| Variant | Program | raw bytes | xz bytes | squashfs bytes |
|---|---|---:|---:|---:|
| MIPS, dynamic musl (as OpenWrt packages) | A floor | 397,108 | 128,223 | 135,168 |
| | B lean | 724,988 | 234,184 | 245,760 |
| | B lean, in-house ubus client | 659,320 | 220,681 | 229,376 |
| | C asyncd | 1,052,808 | 340,577 | 360,448 |
| MIPS, static musl | A floor | 461,836 | 154,401 | 159,744 |
| | B lean | 789,756 | 265,500 | 278,528 |
| | C asyncd | 1,183,116 | 379,358 | 401,408 |
| x86-64 glibc, for comparison | A floor | 297,776 | 129,706 | 131,072 |
| | B lean | 540,352 | 229,317 | 233,472 |
| | C asyncd | 838,608 | 341,599 | 348,160 |

Static linking needed two workarounds: `-C link-self-contained=no`, because `rustc` otherwise
expects the self-contained `crt1.o`, `crti.o`, `crtbegin.o` that only a distributed Tier 2 target
ships; and a `libunwind.a` that points at the toolchain's `libgcc_eh.a`, because `std` links
`-lunwind` and `build-std` does not build one. With both, the three programs link and are
plain (non-PIE) static executables.

### Step 4–5: memory on emulated MIPS

Run target: OpenWrt 25.12.5 `malta/le` (`openwrt-25.12.5-malta-le-vmlinux-initramfs.elf`,
sha256 verified) under `qemu-system-mipsel -M malta -cpu 24Kc -m 128`, kernel 6.12.94, musl
loader `/lib/ld-musl-mipsel-sf.so.1`. The kernel has no `smaps_rollup`, so only RSS is available.
Requests were sent from the host through an SSH tunnel (`curl`), because OpenWrt's `wget` cannot
POST and `uclient-fetch`'s POST did not complete against this toy server.

| Program | Idle, 3 s | Idle, 33 s | After 300 `GET` | After `setpw`, 6 logins (argon2id 8 MiB), `PUT` |
|---|---:|---:|---:|---:|
| A floor, dynamic | 1,124 KiB | 1,124 KiB | — | — |
| A floor, static | 448 KiB | 448 KiB | — | — |
| B lean, dynamic | 1,312–1,364 KiB | same | 1,432 KiB | 1,444 KiB |
| B lean, in-house ubus, dynamic | 1,380 KiB | same | 1,452 KiB | — |
| B lean, static | 660–692 KiB | same | 760 KiB | 768 KiB |
| C asyncd, dynamic | 1,844 KiB | same | 1,884 KiB | — |
| C asyncd, static | 1,128 KiB | same | 1,168 KiB | — |

Idle values vary by a few tens of KiB between runs. The dynamic binaries' extra 0.6–0.7 MB is the
shared musl loader and libc pages, which the system shares with every other process; the static
figures show the daemon's own cost. musl's allocator returns the 8 MiB argon2 working memory after
each hash, so the resident size does not grow with logins (the x86-64 glibc build kept it:
RSS 10.9 MB after the same load).

### Step 6: password check time

3 verifications each. The MIPS column is QEMU's software emulation of a 24Kc on a fast x86-64
host; it is **not** the speed of an MT7628 or MT7621 and can only be settled on a device (M-3).

| Function | Per verify, x86-64 host | Per verify, emulated MIPS 24Kc |
|---|---:|---:|
| argon2id m = 4 MiB, t = 4 | 5 ms | 118 ms |
| argon2id m = 8 MiB, t = 3 | 8 ms | 182 ms |
| argon2id m = 19 MiB, t = 2 | 12 ms | 299 ms |
| pbkdf2-sha256, 100,000 iterations | 22 ms | 532 ms |
| pbkdf2-sha256, 310,000 iterations | 69 ms | 1,874 ms |
| pbkdf2-sha256, 600,000 iterations | 134 ms | 3,280 ms |

argon2id with 8 MiB is 3× cheaper than pbkdf2 at 100,000 iterations on this core and resists
GPU guessing better; it is the candidate for RFC 004 ENK-OWN-011, subject to the device figure.

### Step 7: two ways to reach ubus

| Way | Effort | Size effect (program B, MIPS) |
|---|---|---|
| `std::process::Command` running `ubus call` | Trivial | 724,988 B |
| In-house client on the unix socket (`lookup` only) | About 120 lines; the wire format is 8-byte header + nested length-prefixed attributes | 659,320 B (**66 KB smaller**: `Command` pulls in process-spawning code) |
| Dynamic linking to `libubus` | Not tried: the SDK ships no `libubus` headers or library in `staging_dir`; they would have to be built with the SDK first | — |

## Measured values

| Quantity | Value | Unit | Provisional budget |
|---|---:|---|---:|
| Program B inside squashfs | 240 (224 with in-house ubus) | KiB | ≤ 350 |
| Program C inside squashfs | 352 | KiB | ≤ 350 |
| Program A inside squashfs | 132 | KiB | — |
| Program B resident memory, MIPS, dynamic, idle / loaded | 1.3 / 1.4 | MB | ≤ 3 |
| Program B resident memory, MIPS, static, idle / loaded | 0.7 / 0.8 | MB | ≤ 3 |
| Password check, argon2id 8 MiB t 3, emulated MIPS | 182 | ms | ≤ 500 (device figure pending) |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| Run target: emulated `malta/le` instead of a device | No device at hand. Memory figures transfer; timing does not. |
| Proportional set size not measured | The `malta` kernel has no `smaps_rollup`. |
| Load generated from the host through an SSH tunnel | OpenWrt's `wget` cannot POST; `uclient-fetch --post-data` received no answer from this toy server (not investigated; the production daemon will be tested against real clients). |
| Static variant needed link workarounds | See the size table. OpenWrt packages are dynamic; the static figures are for reference. |
| Toolchain: nightly `build-std` instead of OpenWrt's packages-feed Rust | The feed builds rustc and LLVM from source (hours, tens of GB). Both routes produce dynamic musl binaries against the same libc; sizes should be close, not identical. |

## What contradicts an RFC

None. The numbers fall on both sides of the provisional budget (D-10) as the RFC anticipated:
a blocking daemon fits with room to spare (240 KiB of 350; 1.4 MB of 3 MB), an asynchronous web
framework does not (352 KiB).

## Observations beyond the question

1. `build-std` with the SDK toolchain works out of the box for dynamic binaries: `rustup target add` is not needed, no patches, a one-line `.cargo` linker setting plus `STAGING_DIR`. The whole workspace (three programs, std included) builds in about 2 minutes on this workstation.
2. The dynamic MIPS binaries are PIE and expect `/lib/ld-musl-mipsel-sf.so.1`, which the OpenWrt image provides.
3. `std::process::Command` is expensive on this target (≈ 66 KB raw, 16 KiB in squashfs). A daemon that never spawns processes saves that.
6. On musl the resident size stays flat under password hashing; on glibc it grows by the hash's working memory. Memory budgets must be measured on the target libc.
7. Static linking saves about 0.65 MB of resident memory per process because the loader and libc are not mapped, but the pages of the shared libc are counted once for the whole system; on a device that already runs several musl programs, the dynamic figure overstates the daemon's real cost.
4. The x86-64 and MIPS compressed sizes are nearly equal although the MIPS raw sizes are 30 % larger: MIPS code compresses better.
5. The dependency list of program B includes `proc-macro2`, `quote`, `syn` only at build time (serde derive); they do not reach the binary.

## How to repeat

```sh
cd .git-exclude/tmp/m/m2
./build.sh          # builds every variant; logs in out/<variant>.log
./build.sh sizes    # size table (adds the exact squashfs figure when mksquashfs is installed)
./rss.sh            # host memory and timing
cd ../malta && ./mvm.sh start && ./mssh 'sh /tmp/m2mem.sh' && ./m2load.sh   # emulated MIPS (bin.img carries the binaries)
```
