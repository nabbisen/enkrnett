# Enkrnett

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

**Guest Wi-Fi that is as easy to join as open Wi-Fi and encrypts each visitor's radio link.**

## Overview

Enkrnett (*Enkel* ease + *Vern* protection + *Nett* net) is a deployment and management layer on
OpenWrt for OWE, also called Wi-Fi Enhanced Open. It is made for people who run a shop, café,
restaurant, hotel, or community space and have no IT training: a small daemon on the router, a
phone-first settings screen in English and Japanese, firmware images for supported routers, and
guides. It is not a router operating system and not a cloud service; everything works inside the
venue.

## Why

Open guest Wi-Fi is overheard; password Wi-Fi is a chore. OWE encrypts each guest's radio link
without a password. OpenWrt already supports it, but stock OpenWrt assumes networking knowledge,
places guests on the operator's own network, ships with an empty root password, and shows
configuration rather than what is actually protected. Enkrnett adds what a non-expert needs: an
installation they can complete, a separate and isolated guest network by default, a status that
says exactly what is encrypted and what is not, and a notice for guests that says the same.

## State

Design baseline accepted on 2026-10-31 (RFC 001 to 017). Milestone M1 is under way: the daemon
runs the whole management API against a mock platform, with tests. **No release exists yet.**
Nothing is installable on a router. See [ROADMAP.md](./ROADMAP.md).

## Quick start

For contributors only (nothing runs on a router yet):

```sh
cargo test --workspace
cargo run -q -p enkrnettd -- --mock --listen 127.0.0.1:8080 --state-dir /tmp/enkrnett-demo --act-delay-ms 0
curl -s http://127.0.0.1:8080/api/1/
cd docs && mdbook serve
```

## Design notes

- One device, routed and isolating: guests reach the Internet and nothing else.
- A dedicated management Wi-Fi with a generated key; the settings screen is served by the device
  over HTTP and never from the Internet.
- One owner, one password, one recovery path (Erase).
- Every change is a transaction with a pending marker on flash; the device verifies its own
  state and undoes what fails.
- Two guest modes, "Open to every device" and "Encrypted only", and always two counts.
- Signed full-image updates from the settings screen; nothing is promised that a single-slot
  router cannot do.
- Footprint budgets measured, not estimated: daemon ≤ 300 KiB, UI ≤ 150 KiB, idle ≤ 2 MB.

## More detail

| Where | What |
|---|---|
| [docs/src/specification/](./docs/src/specification/index.md) | The consolidated specification: 241 requirements, threat model, index |
| [rfcs/](./rfcs/README.md) | Decisions with reasons and alternatives (RFC 000 to 017) |
| [docs/src/measurements/](./docs/src/measurements/index.md) | What was measured and what it changed |
| [ROADMAP.md](./ROADMAP.md) | Milestones and order of work |

## License

Apache-2.0. Author: nabbisen.
