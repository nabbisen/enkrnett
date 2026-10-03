# Measurements

Measurements answer questions that no source could settle. They are defined by RFC 006
(`rfcs/<state>/006-feasibility-measurements.md`). Procedures are in
`rfcs/handoffs/006-feasibility-measurements/`.

| ID | Question | State | Report | Review |
|---|---|---|---|---|
| M-1 | What do phone browsers do with a UI served over HTTP from a private address? | Kit prepared; waiting for the owner | — | — |
| M-2 | How large is a daemon of Enkrnett's kind on `mipsel_24kc`, and how much memory does it use? | Ready for review (MIPS run-time numbers pending an emulator) | [report](./m-2-daemon-footprint.md) | — |
| M-3 | Does a complete image fit and run on a `constrained` device? | Blocked: no router; lawful basis pending | — | — |
| M-4 | How do OWE and Transition Mode behave on the candidate devices with real clients? | Blocked: no router; lawful basis pending | — | — |
| M-5 | Can configuration be applied with confirm-or-revert through OpenWrt's own interfaces, by a process that is not `root`? | Ready for review | [report](./m-5-confirm-or-revert.md) | — |
| M-6 | Does OpenWrt in a virtual machine with simulated radios run OWE from association to traffic? | Ready for review | [report](./m-6-simulated-radios.md) | — |
| M-7 | Can non-experts install a candidate device, and recover it, with a written guide alone? | Blocked: no router; lawful basis pending | — | — |
| M-8 | Does the separation of RFC 002 hold against a client that tries to break it? | Blocked: no router; lawful basis pending | — | — |

States: Not started · Blocked · In progress · Ready for review · Accepted · Returned · Superseded.

Reports dated 2026-10-04 were carried out by the architect at the owner's instruction; the
development team's role is noted in each report.
