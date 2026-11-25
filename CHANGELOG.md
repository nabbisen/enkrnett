# Changelog

Releases use semantic versioning; tags carry no prefix (RFC 013 ENK-RLS-001). Until the first
release, the history of decisions is in `rfcs/`.

## Unreleased

### Milestone M1, stage 2 (2026-11-25)

- `ui/`: the management UI, every screen of chapter 10 in English and Japanese; catalogues as
  JSON; the word check and the size check in the build (44 KiB compressed against 150 KiB).
- `tests/browser/`: the Playwright harness, 14 tests at phone size against `enkrnettd --mock`.
- `enkrnettd`: UI files are no longer bounded by the 64 KiB API body limit; `enkrnett-ubus` is
  not a dependency of the daemon until the OpenWrt platform uses it.
- One Japanese sentence of RFC 017 §5.6 corrected by the word check (see `docs/src/design/ui.md` §5).
- CI: UI and browser-harness jobs.

### Milestone M1, stage 1 (2026-10-31)

- Workspace of RFC 013 §5: `enkrnett-model`, `enkrnett-ubus`, `enkrnettd`.
- `enkrnettd`: every resource of the management API against a mock platform; HTTP/1.1 server
  with the limits and headers of chapter 9; ownership state machine; sessions; argon2id; login
  delay; persistence with format version; Erase; static files.
- Tests: unit tests per module, API contract tests over TCP (chapters 3, 4, 9).
- Docs: daemon internal design; building and testing. Example: `claim-flow.sh`.
- Not yet: the UI, the OpenWrt platform, update install, MIPS build.

### Design baseline, 2026-10-31

- RFC 001 to 017 accepted: scope and vocabulary, topology and separation, management channel,
  ownership and sessions, devices and baseline, measurements, threat model, OpenBSD position,
  vocabulary amendment, footprint budgets, settings and changes, guest modes and status, build
  and update, platform findings, management API, amendments from the full review, operator UI.
- Consolidated specification published in `docs/src/specification/` (241 requirements, 44
  threats, 14 remaining risks).
- Measurements M-2, M-5, M-6 complete; M-3 and M-8 part 1 complete on virtual machines.
- No code beyond the Cargo scaffold. Nothing is installable.
