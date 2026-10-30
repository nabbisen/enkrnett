| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-AUTH-001 | A session shall be created only by presenting the owner password. | Test | 004 | 0.1.0 |
| ENK-AUTH-002 | A session shall be identified by a random value of at least 128 bits, which the client sends in a request header. | Inspection | 004 | 0.1.0 |
| ENK-AUTH-003 | A session shall end after 15 minutes without a request, after 12 hours in any case, at logout, at restart, at reset, and when the owner password changes. | Test | 004 | 0.1.0 |
| ENK-AUTH-004 | Session durations shall be measured with a clock that does not depend on the date and time being set. | Test with a wrong system date | 004 | 0.1.0 |
| ENK-AUTH-005 | At most 4 sessions shall exist. Creating another ends the one that has been idle longest. | Test | 004 | 0.1.0 |
| ENK-AUTH-006 | Sessions shall be kept in memory only. | Inspection | 004 | 0.1.0 |
| ENK-AUTH-007 | Without a session the device shall disclose nothing beyond the greeting. In particular it shall not disclose versions of the firmware or its components, the device identifier, or any configuration. | Test of every resource without a session | 004 | 0.1.0 |
| ENK-AUTH-008 | Authorization shall be decided on the device for every request. | Test with a modified client | 004 | 0.1.0 |
