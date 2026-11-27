# Enkrnett

Enkrnett = *Enkel* (ease) + *Vern* (protection) + *Nett* (net).

Enkrnett lets a person without IT training give visitors a Wi-Fi network that is as easy to join
as open Wi-Fi and encrypts each visitor's radio link individually (OWE, also called
WPA3-Enhanced Open).

It is a deployment and management layer on OpenWrt. It is not a router operating system.

## State of the project

Design baseline complete (RFC 001 to 017 accepted, 2026-10-31). Milestones M1 (0.1.0) and M2
(0.2.0) are tagged; M3, *Installable*, is implemented up to its 0.3.0 release candidate
(2026-11-27): signed updates installed from the settings screen, images rebuilt from a lock and
compared, the guides in English. What still needs the owner's Tier 1 router and the legal basis,
the usability test among it, is listed in the release notes. These releases are for development
and testing, not for operators.

| Where | What |
|---|---|
| [Guides](./guides/index.md) | For the person who runs the place: install, set up, update, erase, recover; the expert install |
| `rfcs/` | Decisions and their design, with the reasons and the alternatives |
| [Specification](./specification/index.md) | The accepted RFCs restated with every amendment applied: 241 requirements, the threat model, the requirements index |
| [Measurements](./measurements/index.md) | What was measured, how, and what it changed |
| `ROADMAP.md` | Milestones and order of work |
