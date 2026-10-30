| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-OWE-007 | The guest network shall offer exactly these two modes, named as above in operator-facing text. | Inspection | 012 | 0.2.0 |
| ENK-OWE-008 | The default at setup shall be `open_to_all`. The setup screen shall state both modes' consequence in one line each and shall not pre-select silently: the default is shown as chosen, with the other beside it. | Inspection; M-7 | 012 | 0.2.0 |
| ENK-OWE-009 | In both modes PMF shall be required on the OWE BSS (the platform does this; the daemon shall verify it in the effective state). | Test | 012 | 0.2.0 |
| ENK-OWE-010 | The device shall not send Transition Disable in the first release. | Inspection of the generated configuration | 012 | 0.2.0 |
| ENK-OWE-011 | When the operator switches from `open_to_all` to `encrypted_only`, the UI shall show, before saving, the number of currently connected devices that will lose the network (those not encrypted). | Test | 012 | 0.2.0 |
| ENK-OWE-012 | When the operator switches from `encrypted_only` to `open_to_all`, the UI shall say that encrypted devices may take up to a few minutes to reconnect. | Inspection (M-6 observation 2) | 012 | 0.2.0 |
| ENK-OWE-013 | The effective state shall report the two counts separately and shall never sum them into one "connected" figure without both parts. | Test | 012 | 0.2.0 |
| ENK-OWE-014 | Counts shall be current readings, not stored histories (decision D-15). | Inspection | 012 | 0.2.0 |
| ENK-OWE-015 | The device shall state what it can observe: configured, running, and associated. It shall not claim knowledge of what is on the air. | Inspection of wording | 012 | 0.2.0 |
| ENK-OWE-016 | The status shall show the mode and the two counts on the first screen, with no further tap. | Inspection | 012 | 0.2.0 |
| ENK-OWE-017 | The status shall never use a single badge, colour, or word that summarises protection as good or bad. Colour marks only *running* / *not running*. | Inspection; review against RFC 001 principle 4 | 012 | 0.2.0 |
| ENK-OWE-018 | A "What this protects" link shall open the text of RFC 012 §4, the same text as the guest notice. | Inspection | 012 | 0.2.0 |
| ENK-OWE-019 | The compatibility text shall name the oldest supported system version and hardware generation per platform, and shall be updated from M-4 before the first release. | Release checklist | 012 | 0.2.0 |
