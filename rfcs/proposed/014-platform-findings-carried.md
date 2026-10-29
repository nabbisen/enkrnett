# RFC 014 — Platform findings that Enkrnett must handle itself

**Status.** Proposed
**Tracks.** Results of measurements M-8 part 1 and M-3 part 1 (2026-10-29) that add to the
accepted RFC 002 and RFC 011.
**Touches.** The daemon's apply and health-check steps (RFC 011 §4); firmware defaults; the test
plan of RFC 002.

## Summary

Three behaviours of OpenWrt 25.12.5 found in the virtual machine would silently defeat accepted
requirements if the daemon trusted the platform's configuration options. Each gets a requirement
that makes the daemon set, or verify, the fact itself.

## Findings

| # | Finding | Effect | Measured in |
|---|---|---|---|
| 1 | `bridge_isolate` on a `wifi-iface` sets the kernel's bridge-port `isolated` flag only on the first BSS of a radio. The second and third BSS (the open half of Transition Mode, the management network) stay unisolated, in the automatic and in the two-section form; a `network` device section with `isolate '1'` does not reach them either. Writing the flag directly isolates both directions and keeps Internet access. | A guest on the OWE BSS could reach a guest on the open BSS: ENK-ISO-002 not met. | M-8 rows 6, diagnosis |
| 2 | A second `dnsmasq` instance fails to start on 127.0.0.1:53 unless all but one carry `notinterface loopback`; the failure is visible only in `logread`, and the network without DHCP looks "up". | Guests get no address; status says running. | M-8 runs 2–3 |
| 3 | `uhttpd` and any listener bind to all addresses unless told otherwise. | The management UI would answer on the guest network: ENK-ISO-003 not met. | M-8 setup |

Findings already covered by accepted requirements and repeated here for completeness: the
country must be set on every radio (ENK-CFG-003, M-6); the Transition Mode BSSID fix must be
carried (ENK-CFG-004, M-6).

## Requirements

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-023 | After every Wi-Fi start or reload, the daemon shall set the `isolated` flag on every bridge port of the guest and management bridges (`/sys/class/net/<bridge>/brif/<port>/isolated`), and the health check of ENK-CFG-007 shall read the flags back and fail if any is unset. | Test: M-8 row 6 after reload, after `wifi down/up`, after a restart |
| ENK-CFG-024 | The daemon shall configure one DNS/DHCP instance per Wi-Fi network, bound to that network's interface only, with all but one instance kept off loopback; the health check shall confirm that each instance is running and listening on its network's address. | Test: `netstat -uln` shows one listener per network on 53 and 67; `ubus call service list` |
| ENK-CFG-025 | The daemon shall bind its own HTTP listener to the management address and, in state `SETUP`, the setup address, never to all addresses; the health check shall include a port scan of the device from the guest network and fail on anything but DNS and DHCP. | Test: M-8 row 3 |
| ENK-CFG-026 | The daemon shall treat the platform's configuration options as requests and the kernel's or the service's reported state as the truth: every requirement of RFC 002 that the platform implements shall have a read-back in the health check. | Review of the health check against RFC 002 |

## Consequences

| Affected | Change |
|---|---|
| RFC 011 §4 health check | Gains the three read-backs above. |
| RFC 002 verification | M-8 rows 3 and 6 become release tests, run on the VM in CI and on the Tier 1 device before release. |
| Gap analysis A-11 | Closed: confirmed as a real gap, with the remedy measured. |
| OpenWrt upstream | Worth a report; until fixed, the daemon sets the flag itself (ENK-CFG-023 stays either way, as a read-back). |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Trust `bridge_isolate` and wait for an upstream fix | The failure is silent and defeats a first-release requirement. |
| Separate bridges per BSS instead of the flag | Three bridges, three subnets for one guest network; more to explain and to route. |
| One `dnsmasq` instance with interface-specific options | The management name must not resolve for guests; dnsmasq cannot scope a static address to one interface. |
