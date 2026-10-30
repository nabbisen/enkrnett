| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-TIER-001 | Every supported device shall have a device profile that names the model, the hardware revision, the tier, the device class, and the install class. | Inspection | 005 | 0.1.0 |
| ENK-TIER-002 | A device profile shall declare: the uplink port; the reset button; the light that Enkrnett can control, if any; the radios with their bands and interface limits; whether the platform's hardening is complete on its target; and which hardware address the label prints and how the device obtains that address at run time. | Inspection; test on the device; M-7 (label match) | 005 (amended by 016) | 0.1.0 |
| ENK-TIER-003 | The management UI shall offer only what the device profile and the run-time checks of the device allow. | Test on each profile | 005 | 0.1.0 |
| ENK-TIER-004 | A device shall enter Tier 1 only when RFC 005 §2 is met and the owner has accepted it. | Record in an RFC | 005 | 0.3.0 |
| ENK-TIER-005 | A device on a target with reduced platform hardening shall enter Tier 1 only by a separate owner decision that names this reduction. | Record in an RFC | 005 | 0.3.0 |
