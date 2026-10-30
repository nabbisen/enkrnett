# RFC 010 — Footprint budgets

**Status.** Accepted (2026-10-29)
**Tracks.** Decision D-10 and amendment A-7, with the numbers of measurement M-2.
**Touches.** The daemon's dependency policy; the UI build; firmware profiles; the release
checklist; CI.

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md) and
[RFC 005](./005-devices-baseline-installation.md) (device classes and tiers).

## Summary

Budgets that a release must meet on every Tier 1 device, measured, not estimated. They are set
from M-2 (daemon size and memory on `mipsel_24kc`) and from the stock image sizes of RFC 005.
Two budgets that need a physical `constrained` device (free memory with clients; password-check
time) are stated now and confirmed by M-3.

## Motivation

| Fact | Source |
|---|---|
| A blocking daemon with JSON, argon2, OS randomness and a ubus client measures 240 KiB in squashfs (224 KiB with an in-house ubus client); an axum + tokio daemon measures 352 KiB. | M-2 |
| Resident memory on emulated MIPS: 1.3–1.4 MB dynamic, 0.7–0.8 MB static; argon2's 8 MiB is returned after each hash on musl. | M-2 |
| Stock 25.12.5 images for 8 MB MT7628 devices are 5,824–5,888 KB against limits of 7,360–7,936 KB, and contain LuCI (about 415 KB compressed). | RFC 005; gap analysis F-OWRT-14, F-OWRT-15 |
| OpenWrt calls 8 MB "barely enough". | RFC 005 |
| A target without a budget cannot fail, so it cannot be verified. | Gap analysis GAP-040 |

## Budgets

All sizes are measured as the compressed size inside the squashfs of the release image (the
`mksquashfs` figure), memory as resident set size on the device.

### Flash

| ID | Budget | Constrained class | Standard class | Verification |
|---|---|---:|---:|---|
| ENK-FOOT-001 | Enkrnett daemon, inside squashfs | ≤ 300 KiB | ≤ 300 KiB | CI, every build |
| ENK-FOOT-002 | Management UI bundle (all languages), inside squashfs | ≤ 150 KiB | ≤ 150 KiB | CI, every build |
| ENK-FOOT-003 | Everything else Enkrnett adds (defaults, ACL, scripts, common-password list), inside squashfs | ≤ 50 KiB | ≤ 50 KiB | CI, every build |
| ENK-FOOT-004 | Release image | ≤ the stock OpenWrt image of the same device for the same baseline | same | Release checklist |
| ENK-FOOT-005 | Free overlay after first boot | ≥ 1,200 KiB | ≥ 4,096 KiB | M-3; release checklist |

Budget 001–003 together (500 KiB) are about what LuCI costs in the stock image, which Enkrnett
removes (RFC 002 ENK-HARD-004).

### Memory

| ID | Budget | Value | Verification |
|---|---|---:|---|
| ENK-FOOT-006 | Daemon resident memory, idle, dynamic linking | ≤ 2.0 MB | M-3; release checklist |
| ENK-FOOT-007 | Daemon resident memory after 1,000 requests, outside a password check | ≤ 2.5 MB | M-3; release checklist |
| ENK-FOOT-008 | Daemon resident memory during a password check (transient) | ≤ 12 MB | M-3 |
| ENK-FOOT-009 | Free memory ("available") on a `constrained` device with both networks up and 20 associated clients | ≥ 12 MB | M-3 |

### Time

| ID | Budget | Value | Verification |
|---|---|---:|---|
| ENK-FOOT-010 | One owner-password check on the slowest Tier 1 device | ≤ 500 ms | M-3 on the device (emulation gave 182 ms for argon2id, 8 MiB, t = 3; not representative) |
| ENK-FOOT-011 | Management UI first screen rendered on a 2019 mid-range phone over the management network | ≤ 2 s | Browser harness; M-7 |

## Design constraints that follow

| ID | Constraint | Reason |
|---|---|---|
| ENK-FOOT-012 | The daemon shall use no asynchronous runtime and no web framework. One blocking HTTP/1.1 server with a bounded number of worker threads is sufficient for one operator. | M-2: 352 KiB versus 240 KiB; a single operator never produces concurrency |
| ENK-FOOT-013 | The daemon shall carry no TLS implementation on the device. | RFC 003; M-2 dependency analysis |
| ENK-FOOT-014 | The daemon should reach ubus through its own client on the unix socket rather than by spawning processes. | M-2: `std::process::Command` costs 66 KB raw; spawning is also slower and harder to confine |
| ENK-FOOT-015 | The daemon shall be dynamically linked against the platform's musl, as OpenWrt packages are. | Shares libc pages with the rest of the system; M-2 |
| ENK-FOOT-016 | Every dependency shall be listed with its purpose and its measured size contribution in the daemon's documentation; adding one requires the budget check to pass. | Keeps the list honest over time |
| ENK-FOOT-017 | Password hashing shall use argon2id with parameters chosen so that ENK-FOOT-008 and ENK-FOOT-010 both hold on the slowest Tier 1 device; the parameters are recorded with the device profile. | M-2: argon2id (8 MiB) is 3× cheaper than pbkdf2 at 100,000 iterations and resists GPU guessing |

## How the budgets are enforced

| When | Check |
|---|---|
| Every CI build | Sizes of the daemon, the UI bundle, and the extras against ENK-FOOT-001–003; the build fails when exceeded |
| Every release | ENK-FOOT-004 and 005 per Tier 1 device; memory and time figures from the latest device measurement, recorded in the release notes |
| A new Tier 1 device | All budgets measured on it before promotion (RFC 005 §2 criterion 7) |

## Consequences

| Affected | Change |
|---|---|
| D-10 | Decided by this RFC. |
| RFC 005 §2 criterion 7 | Points to these budgets. |
| RFC 006 §3 provisional thresholds | Superseded. M-3's procedure measures ENK-FOOT-005 to 010. |
| Gap analysis A-01, A-03, A-12 | Closed by M-2 (A-02 waits for M-3). |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Budgets as fractions of device flash and RAM | Fractions hide the absolute cost; the 8 MB device is the one that matters, and it has fixed numbers. |
| No UI budget, "the UI is static files" | A Svelte bundle with two languages and icons can quietly reach 400 KiB. |
| Static linking to save memory | Saves 0.65 MB per process but the shared libc is counted once for the whole system; dynamic is what every other OpenWrt program does. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Confirmation of ENK-FOOT-005 to 010 on a `constrained` device | M-3 |
| 2 | argon2id parameters per device profile | M-3 on each Tier 1 device |
