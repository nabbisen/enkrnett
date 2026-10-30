| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-HARD-001 | No account shall allow login without a credential that was generated for, or chosen on, that device. | Inspection of the image; login attempts | 002 | 0.2.0 |
| ENK-HARD-002 | No remote shell shall be reachable on any network. | Port scan on every network | 002 | 0.2.0 |
| ENK-HARD-003 | Failsafe mode shall give no network access to the device. | Test: trigger failsafe, scan | 002 | 0.2.0 |
| ENK-HARD-004 | No general-purpose administration interface shall be installed. | Inspection of the package list | 002 | 0.2.0 |
| ENK-HARD-005 | The image shall contain no network service that RFC 002 §3 does not list. | Comparison of listening sockets with RFC 002 §3 | 002 | 0.2.0 |
| ENK-HARD-006 | The device shall report, for each of ENK-HARD-001 to 005 and ENK-HARD-008, whether it is met. | Test on a modified device | 002 (amended by 016) | 0.2.0 |
| ENK-HARD-007 | A development build may deviate from this section. It shall identify itself as a development build on every page of the management UI and shall not be published as a release. | Inspection | 002 | 0.1.0 |
| ENK-HARD-008 | WPS shall not be available on any network. | Inspection of the image; test | 007 (amended by 007) | 0.2.0 |
