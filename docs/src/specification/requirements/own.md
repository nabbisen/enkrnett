| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-OWN-001 | A device without an owner shall open the setup window when it finishes starting. Nothing else shall open it. | Test | 004 | 0.1.0 |
| ENK-OWN-002 | The setup window shall stay open for 30 minutes. Time spent in `PENDING` does not count. If a claim session exists when the 30 minutes end, or the device has returned from `PENDING`, the window shall stay open for 15 minutes more, once. | Test | 004 | 0.1.0 |
| ENK-OWN-003 | When the window closes, the device shall stop offering any Wi-Fi network. | Test | 004 | 0.1.0 |
| ENK-OWN-004 | The setup network's name shall end with the last four characters of the hardware address printed on the device's label. The device profile names which address the label prints and how the device obtains it (ENK-TIER-002). | Test per device profile | 004 (amended by 016) | 0.1.0 |
| ENK-OWN-005 | At most one claim session shall exist. It shall end after 5 minutes without a request from its client. | Test with two clients | 004 | 0.1.0 |
| ENK-OWN-006 | Where the device profile declares a light that Enkrnett can control, the claim shall include the identify step. Where it declares none, the UI shall say that the step is not available on this device. | Test per device profile | 004 | 0.1.0 |
| ENK-OWN-007 | The device shall change nothing that persists until the operator confirms in step 7. | Test: abandon at every step, restart, inspect | 004 | 0.1.0 |
| ENK-OWN-008 | A claim shall become final only when the owner has logged in through the management network. If this has not happened 10 minutes after step 8, or if the device restarts before it, the device shall forget everything the claim created and return to `SETUP`. | Test, including power loss after step 8 | 004 | 0.1.0 |
| ENK-OWN-009 | The owner password shall have 10 to 64 characters from the printable ASCII set. | Test | 004 | 0.1.0 |
| ENK-OWN-010 | The device shall refuse an owner password that equals the name of one of its networks, the management key, or the device identifier, or that appears in the list of common passwords shipped with the release. | Test | 004 | 0.1.0 |
| ENK-OWN-011 | The device shall keep the owner password only as a salted hash made with a function designed to be slow. It shall never return, display, or log the password. | Inspection; search of all outputs | 004 | 0.1.0 |
| ENK-OWN-012 | After 5 consecutive failed logins the device shall refuse further attempts for 30 seconds, doubling after each further failure, up to 15 minutes. A successful login or a restart ends the delay. | Test | 004 | 0.1.0 |
| ENK-OWN-013 | The owner shall be able to change the owner password by entering the current one. The change ends every session. | Test | 004 | 0.1.0 |
| ENK-OWN-014 | The device shall generate the management key. The operator does not choose it. | Inspection | 004 | 0.1.0 |
| ENK-OWN-015 | The management key shall carry at least 80 bits of randomness, shall have at most 24 characters, and shall use no characters that are easily mistaken for one another. | Inspection of the generator; statistical test | 004 | 0.1.0 |
| ENK-OWN-016 | The owner shall be able to display the management key and to replace it with a new one. A replacement becomes final as ENK-API-021 and ENK-API-022 define, within 600 s; otherwise the previous key returns. | Test | 004 (amended by 015, 016) | 0.1.0 |
| ENK-OWN-017 | Erase shall be started by holding the reset button as the platform defines, or by the owner in the management UI after entering the owner password again. | Test per device profile | 004 (amended by 009) | 0.1.0 |
| ENK-OWN-018 | Erase shall remove the owner password, the management key, the settings, every session, the device identifier, and every log. Nothing that the operator or the device created shall remain. | Inspection after reset | 004 (amended by 009, 016) | 0.1.0 |
| ENK-OWN-019 | Erase shall be the only way to end ownership. | Inspection | 004 (amended by 009) | 0.1.0 |
| ENK-OWN-020 | The device shall create a new random device identifier when it has none. The identifier is not a secret. | Test | 004 | 0.1.0 |
| ENK-OWN-021 | Enkrnett shall keep the platform's meaning of each button and shall assign no further meaning to the duration of a press. The physical button keeps the platform's name "reset button" in device profiles. | Test per device profile | 004 (amended by 009) | 0.1.0 |
| ENK-OWN-022 | In state `SETUP`, before the operator enters any value, the management UI shall state whether the operator's phone is connected to the setup network with or without encryption. | Test with a client that supports OWE and one that does not | 007 (amended by 007) | 0.1.0 |
