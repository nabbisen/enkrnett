| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-RF-001 | Setup shall establish the country before the management or guest network starts. | Test | 005 | 0.1.0 |
| ENK-RF-002 | Where a device profile belongs to one market, the profile shall fix the country and the operator shall not be asked. | Test | 005 | 0.1.0 |
| ENK-RF-003 | Before the country is established the device shall transmit only on channels and at power levels that the platform's regulatory database allows in every country. | Test | 005 | 0.1.0 |
| ENK-RF-004 | Enkrnett shall offer no setting for transmit power, and no setting that selects a channel or a behavior outside what the platform's regulatory database allows for the country. | Inspection | 005 | 0.1.0 |
| ENK-RF-005 | The project shall publish no prebuilt image for a device of a market until the legal basis for operating that device with replaced firmware in that market is documented in `docs/src/`. | Release checklist | 005 | 0.3.0 |
