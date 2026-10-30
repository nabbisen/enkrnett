# 12. Threat model

From RFC 007, as amended by RFC 016 (controls of threats 23, 37, 38, 39, 41; threats 43, 44;
R-14). Accepting a release means accepting the remaining risks of §12.6. The model is updated by
an RFC at every release and whenever an RFC changes a data flow, an integration, or
authorization; threat numbers are never reused.

## 12.1 Scope and assumptions

| In scope | Out of scope (reason) |
|---|---|
| The device as chapters 2 to 11 define it; the management UI and API; claim, ownership, sessions, Erase; the guest, management, and setup networks; buttons, power, Ethernet ports; the update path | The site network itself (belongs to the operator); guests' own devices; what the site router and the Internet do with guest traffic; attacks that open the case |

| Assumption | If it does not hold |
|---|---|
| OpenWrt and its components behave as documented. | Threat 36. |
| The operator's phone is not under an attacker's control. | Threat 42. |
| The operator keeps the management key private. | Threats 4, 6, 7. |

## 12.2 What is protected

| ID | Asset | Harm if lost |
|---|---|---|
| AS-1 | Ownership of the device | Someone else decides how the device behaves |
| AS-2 | Owner password | Same |
| AS-3 | Management key | The management network is open to a stranger |
| AS-4 | Settings | The guest network is not what the operator chose |
| AS-5 | Guests' traffic on the radio link | Guests are overheard |
| AS-6 | The site network | The operator's business devices are reached |
| AS-7 | Guests' devices | A guest is attacked by another guest |
| AS-8 | Availability of the guest network | Guests have no Wi-Fi |
| AS-9 | Truth of what Enkrnett says | Operator or guests believe in protection that is not there |
| AS-10 | The device's software | The device does what an attacker wants |
| AS-11 | Guests' privacy | Guests are tracked |

## 12.3 Who attacks

| ID | Attacker | Position and ability |
|---|---|---|
| AC-1 | Guest | Connected to the guest network. Sends any traffic. |
| AC-2 | Listener | Within radio range. Records. |
| AC-3 | Nearby sender | Within radio range. Sends frames; runs a fake network. |
| AC-4 | Key holder | Knows the management key, not the owner password. |
| AC-5 | Site device | A device on the site network under an attacker's control. |
| AC-6 | Web page | A page open in the operator's browser. |
| AC-7 | Internet host | Anywhere on the Internet. |
| AC-8 | Person at the device | Power, buttons, ports. |
| AC-9 | Operator's phone | Malware on the phone the operator manages with. |
| AC-10 | Supplier | Can alter an image, a package, or an input of the build. |

## 12.4 Where attacks enter

Guest network (OWE BSS and, in Transition Mode, open BSS); management network; setup network;
DHCP and DNS on the device; management UI and API; the uplink (outgoing connections only: DHCP,
and the daily index fetch); power, buttons, Ethernet ports; the operator's browser.

## 12.5 Threats

Verification "Test" means a test derived from the named requirement; "M-n" a measurement.

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 1 | Claims the device during the setup window before the operator does | AC-3 | AS-1 | ENK-OWN-001, 004, 005, 008 | Test | R-1 |
| 2 | Runs a fake setup network and relays the operator's setup | AC-3 | AS-1, 2, 3 | ENK-OWN-004, 006; ENK-TIER-002 (label address) | Test; M-7 | R-2 |
| 3 | Overhears what the operator enters during setup | AC-2 | AS-2, 3 | OWE on the setup network; ENK-OWN-022 | Test | R-3 |
| 4 | Guesses the owner password over the network | AC-4 | AS-2 | ENK-OWN-009, 010, 012 | Test | R-4 |
| 5 | Guesses the management key from a recording | AC-2 | AS-3 | ENK-OWN-014, 015 | Inspection | None known |
| 6 | Reads the owner's session on the management network | AC-4 | AS-2 | WPA3-Personal where the phone supports it; `connection.security` tells the owner | Test | R-5 |
| 7 | Uses a session that belongs to the owner | AC-4, 9 | AS-1 | ENK-AUTH-002, 003, 005, 006 | Test | Until the session ends |
| 8 | Finds a secret in a log, a status, or a diagnosis | AC-4, 8 | AS-2, 3 | ENK-OWN-011; ENK-SEC-016; ENK-API-030 | Search of every output | None known |
| 9 | Finds a secret in the browser's cache | AC-9 | AS-3 | ENK-SEC-015; ENK-API-016 | Test | None known |
| 10 | Sends requests to the device in the operator's name | AC-6 | AS-1, 4 | ENK-SEC-010, 011; ENK-API-002 (a body on every POST) | Test from another origin | None known |
| 11 | Reaches the device under a foreign name (DNS rebinding) | AC-6 | AS-1, 4 | ENK-SEC-009 | Test | None known |
| 12 | Shows the management UI inside another page | AC-6 | AS-4 | ENK-SEC-012; ENK-API-025 | Test | None known |
| 13 | Supplies a value that the management UI runs as script | AC-1 | AS-1 | ENK-SEC-012, 013; ENK-UI-022 | Test with hostile values | None known |
| 14 | Reaches the management UI or API from the guest network | AC-1 | AS-1 | ENK-ISO-003, ENK-NET-009, ENK-CFG-025 | M-8; Test | None known |
| 15 | Reaches a platform service from the guest network | AC-1 | AS-10 | ENK-ISO-003; ENK-HARD-002, 004, 005 | M-8; Test | None known |
| 16 | Reaches the device from the site network | AC-5 | AS-1, 10 | ENK-ISO-005 | M-8; Test | None known |
| 17 | Reaches the device from the Internet | AC-7 | AS-1, 10 | ENK-ISO-005; ENK-TOPO-001 | Test | None known |
| 18 | Reaches the device over IPv6 | AC-1, 5 | AS-1, 10 | ENK-NET-008 | M-8; Test | None known |
| 19 | Joins a network through WPS | AC-3, 8 | AS-3 | ENK-HARD-008 | Inspection; Test | None known |
| 20 | Obtains a shell through failsafe mode | AC-8 | AS-10 | ENK-HARD-003 | Test | None known |
| 21 | Plugs into a free Ethernet port | AC-8 | AS-6 | ENK-TOPO-004 | Test | None known |
| 22 | Makes a value from a request reach a command interpreter | AC-4, 3 | AS-10 | ENK-SEC-014 | Inspection; Test | None known |
| 23 | Exhausts the daemon with large, malformed, or slow requests | AC-4, 3 | AS-8 | ENK-SEC-017 to 022 | Test | None known beyond R-10 |
| 24 | Reaches a device on the site network | AC-1 | AS-6 | ENK-ISO-001, 006 | M-8; Test | None known |
| 25 | Reaches another guest through the device | AC-1 | AS-7 | ENK-ISO-002; ENK-CFG-023 | M-8; Test | None known |
| 26 | Sends broadcast frames to other guests with the shared group key | AC-1 | AS-7 | None | M-8 | R-6 |
| 27 | Answers other guests' requests for an address or for the router | AC-1 | AS-7 | ENK-ISO-002 | M-8 | Through R-6 |
| 28 | Reaches the site network or a guest from the management network | AC-4 | AS-6, 7 | ENK-ISO-004 | M-8; Test | None known |
| 29 | Listens to a guest's traffic | AC-2 | AS-5 | OWE | M-4 | R-7 |
| 30 | Runs a fake access point with the guest network's name | AC-3 | AS-5 | None possible | — | R-8 |
| 31 | Pushes a client that supports OWE onto the open network | AC-3 | AS-5 | PMF (ENK-OWE-009) | M-4 | R-9 |
| 32 | Disturbs the Wi-Fi by interference or a flood of connection attempts | AC-3 | AS-8 | None | — | R-10 |
| 33 | Erases the device and claims it | AC-8 | AS-1, 4 | None; Erase by button is intended (ENK-OWN-017) | — | R-11 |
| 34 | Restarts the device or removes power | AC-8 | AS-8 | None | — | R-11 |
| 35 | Opens the case | AC-8 | All | Out of scope | — | — |
| 36 | Uses a flaw in a platform component that answers on the guest network | AC-1 | AS-10 | ENK-HARD-005; ENK-BASE-002, 003 | Inspection | R-12 |
| 37 | Uses a flaw in the Enkrnett daemon | AC-4, 3 | AS-10 | Memory-safe language; ENK-SEC-014; ENK-CFG-019 to 022 | Review; inspection | R-12 on targets with reduced hardening |
| 38 | Delivers an altered image, package, or update | AC-10 | AS-10 | ENK-RLS-004, 005, 007; ENK-UPD-003, 009 | Test | R-14 |
| 39 | Enkrnett states more protection than exists | — | AS-9 | ENK-SCOPE-004; ENK-OWE-013 to 018; ENK-GUEST-002 | Review of every status and message | None known |
| 40 | The UI shows a state from before an update | — | AS-9 | ENK-MGMT-005; ENK-API-024 | Test | None known |
| 41 | The device keeps records of guests | — | AS-11 | D-15: ENK-OWE-014; ENK-API-028 to 030 | Inspection | None known |
| 42 | The operator's phone is under an attacker's control | AC-9 | AS-1 | None beyond ENK-AUTH-003 | — | R-13 |
| 43 | Replays an old, validly signed release index so that the device offers an older release as an update | AC-5, 7 | AS-10 | ENK-UPD-009 | Test with a replayed index | None known |
| 44 | Delivers, or the project publishes, an image that passes every check and does not boot on a single-slot device | AC-10; project error | AS-8, 10 | ENK-UPD-005 for images that boot; release checklist item 2 | Release checklist | R-14 |

## 12.6 Risks that remain

| ID | Risk | Who | Effect | How it is noticed or reduced | What would close it |
|---|---|---|---|---|---|
| R-1 | The device is claimed by someone else during the setup window. | AC-3 | The operator cannot finish setup. | Setup fails visibly. Erase and repeat. | A secret given to the device at installation |
| R-2 | A relayed fake setup network learns the owner password and the management key. | AC-3 | Full control, unnoticed. | Needs presence and preparation at the moment of setup; the label suffix. | Same, with a password-authenticated key exchange |
| R-3 | Setup from a phone without OWE can be overheard. | AC-2 | As R-2. | The UI says so before anything is entered. | Same |
| R-4 | Repeated wrong passwords keep the owner from logging in. | AC-4 | No management until the delay ends. | Erase ends it. Replacing the management key excludes the attacker. | — |
| R-5 | With WPA2, a key holder can decrypt another client's session, including the owner password. | AC-4 | Full control. | The key is private; the UI tells the owner when their phone used WPA2; phones that use WPA3 are not affected. | Password-authenticated login; WPA3 only |
| R-6 | A guest can send broadcast frames to other guests. | AC-1 | Depends on the victim's device. | Stated to the operator and in the notice. | Per-client group keys, where the platform offers them |
| R-7 | In Transition Mode, clients without OWE are not encrypted. | AC-2 | Those guests are overheard. | The UI shows how many. | Encrypted only |
| R-8 | A fake access point with the same name is accepted by guests' devices. | AC-3 | Those guests' traffic passes through the attacker. | Stated to operator and guests. | Not possible with OWE |
| R-9 | A client that supports OWE is pushed onto the open network. | AC-3 | That guest is overheard. | PMF stops forged disconnects. | Encrypted only; Transition Disable |
| R-10 | The Wi-Fi is disturbed. | AC-3 | No service while it lasts. | — | Not possible |
| R-11 | A person at the device restarts it, or erases and claims it. | AC-8 | No service; or a device under that person's control. | The operator's management key stops working. Placement out of reach. | Placement |
| R-12 | A flaw in a platform component or in the daemon is used before an update is installed; greater on targets with reduced hardening. | AC-1, 4 | Up to full control. | Few services; non-root daemon; updates. | — |
| R-13 | The operator's phone is under an attacker's control. | AC-9 | Full control while the owner is logged in. | Sessions end after 15 minutes without use. | — |
| R-14 | An update image does not boot on a single-slot device. | AC-10; project error | No service; the device needs the recovery procedure of its device profile. | Every release is installed on every Tier 1 device before publication; the UI says the update cannot be undone from the device. | Devices with two firmware slots |
