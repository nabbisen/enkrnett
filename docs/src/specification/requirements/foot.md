| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-FOOT-001 | Enkrnett daemon, inside squashfs: constrained class ≤ 300 KiB, standard class ≤ 300 KiB | CI, every build | 010 | 0.1.0 |
| ENK-FOOT-002 | Management UI bundle (all languages), inside squashfs: constrained class ≤ 150 KiB, standard class ≤ 150 KiB | CI, every build | 010 | 0.1.0 |
| ENK-FOOT-003 | Everything else Enkrnett adds (defaults, ACL, scripts, common-password list), inside squashfs: constrained class ≤ 50 KiB, standard class ≤ 50 KiB | CI, every build | 010 | 0.1.0 |
| ENK-FOOT-004 | Release image: constrained class ≤ the stock OpenWrt image of the same device for the same baseline, standard class same | Release checklist | 010 | 0.3.0 |
| ENK-FOOT-005 | Free overlay after first boot: constrained class ≥ 1,200 KiB, standard class ≥ 4,096 KiB | M-3; release checklist | 010 | 0.4.0 |
| ENK-FOOT-006 | Daemon resident memory, idle, dynamic linking: ≤ 2.0 MB | M-3; release checklist | 010 | 0.3.0 |
| ENK-FOOT-007 | Daemon resident memory after 1,000 requests, outside a password check: ≤ 2.5 MB | M-3; release checklist | 010 | 0.3.0 |
| ENK-FOOT-008 | Daemon resident memory during a password check (transient): ≤ 12 MB | M-3 | 010 | 0.3.0 |
| ENK-FOOT-009 | Free memory ("available") on a `constrained` device with both networks up and 20 associated clients: ≥ 12 MB | M-3 | 010 | 0.4.0 |
| ENK-FOOT-010 | One owner-password check on the slowest Tier 1 device: ≤ 500 ms | M-3 on the device (emulation gave 182 ms for argon2id, 8 MiB, t = 3; not representative) | 010 | 0.3.0 |
| ENK-FOOT-011 | Management UI first screen rendered on a 2019 mid-range phone over the management network: ≤ 2 s | Browser harness; M-7 | 010 | 0.1.0 |
| ENK-FOOT-012 | The daemon shall use no asynchronous runtime and no web framework. One blocking HTTP/1.1 server with a bounded number of worker threads is sufficient for one operator. Reason: M-2: 352 KiB versus 240 KiB; a single operator never produces concurrency | Inspection; CI | 010 | 0.1.0 |
| ENK-FOOT-013 | The daemon shall carry no TLS implementation, and the management channel shall use none. The platform's downloader may use the platform's TLS for the release index and images. | Inspection; CI | 010 (amended by 016) | 0.1.0 |
| ENK-FOOT-014 | The daemon should reach ubus through its own client on the unix socket rather than by spawning processes. Reason: M-2: `std::process::Command` costs 66 KB raw; spawning is also slower and harder to confine | Inspection; CI | 010 | 0.1.0 |
| ENK-FOOT-015 | The daemon shall be dynamically linked against the platform's musl, as OpenWrt packages are. Reason: Shares libc pages with the rest of the system; M-2 | Inspection; CI | 010 | 0.1.0 |
| ENK-FOOT-016 | Every dependency shall be listed with its purpose and its measured size contribution in the daemon's documentation; adding one requires the budget check to pass. Reason: Keeps the list honest over time | Inspection; CI | 010 | 0.1.0 |
| ENK-FOOT-017 | Password hashing shall use argon2id with parameters chosen so that ENK-FOOT-008 and ENK-FOOT-010 both hold on the slowest Tier 1 device; the parameters are recorded with the device profile. Reason: M-2: argon2id (8 MiB) is 3× cheaper than pbkdf2 at 100,000 iterations and resists GPU guessing | Inspection; CI | 010 | 0.1.0 |
