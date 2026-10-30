# Changelog

Releases use semantic versioning; tags carry no prefix (RFC 013 ENK-RLS-001). Until the first
release, the history of decisions is in `rfcs/`.

## Unreleased

### Design baseline, 2026-10-31

- RFC 001 to 017 accepted: scope and vocabulary, topology and separation, management channel,
  ownership and sessions, devices and baseline, measurements, threat model, OpenBSD position,
  vocabulary amendment, footprint budgets, settings and changes, guest modes and status, build
  and update, platform findings, management API, amendments from the full review, operator UI.
- Consolidated specification published in `docs/src/specification/` (241 requirements, 44
  threats, 14 remaining risks).
- Measurements M-2, M-5, M-6 complete; M-3 and M-8 part 1 complete on virtual machines.
- No code beyond the Cargo scaffold. Nothing is installable.
