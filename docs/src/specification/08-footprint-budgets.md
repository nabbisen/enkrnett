# 8. Footprint budgets

From RFC 010, as amended by RFC 016 (ENK-FOOT-013). Numbers come from measurement M-2
(`docs/src/measurements/m-2-daemon-footprint.md`) and the stock image sizes of chapter 5.

All sizes are measured as the compressed size inside the squashfs of the release image (the
`mksquashfs` figure), memory as resident set size on the device. Budgets are enforced by CI on
every build (sizes) and by a named measurement before a release (memory, time, device figures).

## 8.1 Budgets and constraints

{{#include requirements/foot.md}}

Budgets 001–003 together (500 KiB) are about what LuCI costs in the stock image, which Enkrnett
removes (ENK-HARD-004).

## 8.2 How they are enforced

| When | Check |
|---|---|
| Every CI build | ENK-FOOT-001 to 003; the build fails when exceeded |
| Every release | ENK-FOOT-004 and 005 per Tier 1 device; memory and time figures from the latest device measurement, recorded in the release notes |
| A new Tier 1 device | Every budget measured on it before promotion (chapter 5 criterion 7) |

## 8.3 Measured so far

| Quantity | Value | Where |
|---|---|---|
| Blocking daemon with JSON, argon2, OS randomness, ubus client, squashfs | 240 KiB (224 KiB with an in-house ubus client) | M-2 |
| axum + tokio daemon, squashfs | 352 KiB | M-2 |
| Resident memory on emulated MIPS, dynamic | 1.3–1.4 MB | M-2 |
| argon2id 8 MiB, t = 3, emulated | 182 ms (not representative of real hardware) | M-2 |
| Image for the two constrained candidates | 5,696 KB against stock 5,888 KB | M-3 part 1 |

## 8.4 Open

| # | Question | Closed by |
|---|---|---|
| 1 | ENK-FOOT-005 to 010 on a physical `constrained` device | M-3 part 2 |
| 2 | argon2id parameters per device profile | M-3 on each Tier 1 device |
