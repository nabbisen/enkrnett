| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-BUILD-007 | The repository shall contain a lock of every input of a release: the Image Builder archive and its hash; the list of OpenWrt packages with versions and hashes; the Rust toolchain and the Cargo lock; the UI package lock; the carried patches (RFC 011 ENK-CFG-004). | Inspection | 013 | 0.3.0 |
| ENK-BUILD-008 | OpenWrt packages shall be taken from the project's pinned mirror, never from the live feeds, when a release is built. | CI configuration | 013 | 0.3.0 |
| ENK-BUILD-009 | Two builds of the daemon and of the UI bundle from the same lock shall be bit-identical; CI shall build twice and compare. | CI | 013 | 0.1.0 |
| ENK-BUILD-010 | Two builds of an image from the same lock should be bit-identical; CI shall compare and the release notes shall state the result. | CI; release notes | 013 | 0.3.0 |
| ENK-BUILD-011 | CI shall enforce the budgets of RFC 010 on every build. | CI | 013 | 0.1.0 |
| ENK-BUILD-012 | Every release shall publish, beside the images: the lock, the hashes, the signatures, and the written offer of source for the GPL components (the OpenWrt tag and feed revisions of the lock, and where to obtain them). | Release checklist | 013 | 0.3.0 |
