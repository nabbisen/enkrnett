| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-BASE-001 | An Enkrnett release shall be built on exactly one OpenWrt baseline. The first is 25.12. | Inspection | 005 | 0.3.0 |
| ENK-BASE-002 | Enkrnett shall publish releases for a baseline only while OpenWrt maintains that baseline. | Release checklist | 005 | 0.3.0 |
| ENK-BASE-003 | Before the first release, the project shall define and test how an installed device is updated, and how it is upgraded to the next baseline. | Decision D-11; test | 005 | 0.3.0 |
| ENK-BASE-004 | Update and upgrade shall keep the owner, the management key, and the settings. | Test | 005 (amended by 016) | 0.3.0 |
| ENK-BASE-005 | Everything Enkrnett stores shall carry a format version, so that a newer release can read what an older one wrote. | Inspection; test with stored data of every earlier release | 005 | 0.3.0 |
| ENK-BASE-006 | No release shall leave an installed Tier 1 device without a path to the release that follows it. | Release checklist | 005 | 0.3.0 |
| ENK-BASE-007 | When a new baseline no longer builds a Tier 1 device, operators of that device shall be told in the management UI at least one release before support ends. | Release checklist | 005 | 0.3.0 |
