# Device profiles

One directory per supported device model and hardware revision (chapter 5; ENK-TIER-001, 002).
`profile.json` is read by the build (which Image Builder profile, which packages), by the daemon
(uplink port, reset button, light, radios, label address, fixed country, argon2 parameters), and
by the release checklist (tier, verification). The schema is `enkrnett_model::profile`, and the
model crate's tests parse every profile in this directory.

| Field | Meaning |
|---|---|
| `tier` | 1, 2, or 3 (chapter 5 §1). A profile whose `verified.on_hardware` is false is at most Tier 2. |
| `device_class` | `standard` or `constrained` |
| `target`, `package_arch`, `image_builder_profile` | OpenWrt target, package architecture, and the Image Builder's `PROFILE=` name |
| `install` | `self_install` or `expert_install`; the vendor image kind; the vendor firmware versions the installation was tested from (ENK-INST-003) |
| `flash_image_limit_kib`, `ram_mib` | From the target's image definitions and the device page |
| `update` | What the release index states for this device (ENK-RLS-006): `min_free_kib`, the free memory an installation needs beyond the image (ENK-UPD-003), and `install_seconds`, the duration the confirmation screen names (RFC 017 §5.6) |
| `uplink_port` | The Ethernet port that carries the uplink (ENK-TOPO-004) |
| `reset_button` | The platform's name of the button (ENK-OWN-021) |
| `light` | The LED Enkrnett may blink for the identify step, or `null` (ENK-OWN-006) |
| `radios` | Band, chip, and interface limit per radio (chapter 5 §2 criterion 5) |
| `platform_hardening_complete` | Whether the target keeps `ujail` and PIE (chapter 2 §6; ENK-TIER-005) |
| `label_address` | Which hardware address the label prints and where the device reads it (ENK-TIER-002 as amended; ENK-OWN-004) |
| `country` | `fixed` for a device sold in one market (ENK-RF-002), else `null` |
| `argon2` | The parameters that meet ENK-FOOT-008 and 010 on this device (ENK-FOOT-017) |
| `image` | Packages removed from and added to the Image Builder's default set |
| `verified` | Whether a physical unit confirmed the hardware fields, and which measurements do |

Both profiles here describe candidates from chapter 5 §6; none has been verified on a unit.
