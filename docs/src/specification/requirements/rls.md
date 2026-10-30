| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-RLS-001 | Releases shall use semantic versioning; tags carry no prefix. Each release names exactly one OpenWrt baseline (ENK-BASE-001). | Inspection | 013 | 0.3.0 |
| ENK-RLS-002 | Image files shall be named `enkrnett-<version>-<baseline>-<device>-<kind>.bin`, where `<kind>` is `sysupgrade` or the vendor-install kind of the device profile. | Inspection | 013 | 0.3.0 |
| ENK-RLS-003 | The status screen shall show the installed Enkrnett version and the baseline in one line, in words the operator can read back to a helper: "Enkrnett 1.2 on OpenWrt 25.12". | Inspection | 013 | 0.3.0 |
| ENK-RLS-004 | Every published image shall be accompanied by a detached signature made with the project's release key; the release index shall be signed with the same key. | Release checklist | 013 | 0.3.0 |
| ENK-RLS-005 | The image shall embed the public keys that may sign its successor: the current key and one designated next key. A release signed by the next key makes it current and designates a further next key. | Test: rotate once in CI | 013 | 0.3.0 |
| ENK-RLS-006 | The release index shall be one small signed document listing, per device profile and baseline, the latest release: version, file name, size, hash, minimum free memory, release notes URL, whether it changes the baseline, and a sequence number that increases with every publication. | Inspection | 013 (amended by 016) | 0.3.0 |
| ENK-RLS-007 | The signing key shall never be on a CI runner. Signing is a manual step on the project's signing host (RFC 008). | Release checklist | 013 | 0.3.0 |
