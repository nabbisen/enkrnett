# RFC 007 — Threat model

**Status.** Accepted (2026-09-30)
**Tracks.** Who may attack what, by which path, what stops them, and what remains.
**Touches.** RFC 002, 003, 004 (adds six requirements); test plans; the release checklist.

Terms are defined in [RFC 001](../accepted/001-scope-people-vocabulary.md).

## Summary

| Content | Count |
|---|---:|
| Things that are protected | 11 |
| Kinds of attacker | 10 |
| Threats | 42 |
| Requirements added, where the accepted RFCs had no control | 6 |
| Risks that remain | 13 |

**Accepting this RFC means accepting the 13 remaining risks of section 7.**

## Motivation

The project rules require a security review at every release, and an update of the threat model
whenever data flows, integrations, or authorization change. A model must exist before it can be
updated. Tests of security behavior are derived from it.

## 1. Scope

| In scope | |
|---|---|
| The device as RFC 002 to RFC 005 define it | |
| The management UI and the management API | |
| Claim, ownership, sessions, reset | |
| The guest, management, and setup networks | |
| Buttons, power, and Ethernet ports | |

| Out of scope | Reason |
|---|---|
| Security of the site network itself | It belongs to the operator. Enkrnett keeps guests out of it and does not manage it. |
| Security of guests' own devices | Enkrnett cannot control them. |
| What the site router and the Internet do with guest traffic | OWE protects the radio link only ([RFC 8110][rfc8110] §7). |
| Attacks that need opening the device's case | Whoever can do that controls the device. |
| Build, release, and update | They get their own RFC. Their threats are listed here as placeholders. |

| Assumption | If it does not hold |
|---|---|
| OpenWrt and its components behave as their documentation and source say. | Threat 36. |
| The operator's phone is not under an attacker's control. | Threat 42. |
| The operator keeps the management key private. | Threats 4, 6, 7. |

## 2. What is protected

| ID | Asset | Harm if lost |
|---|---|---|
| AS-1 | Ownership of the device | Someone else decides how the device behaves |
| AS-2 | Owner password | Same |
| AS-3 | Management key | The management network is open to a stranger |
| AS-4 | Wi-Fi configuration | The guest network is not what the operator chose |
| AS-5 | Guests' traffic on the radio link | Guests are overheard |
| AS-6 | The site network | The operator's business devices are reached |
| AS-7 | Guests' devices | A guest is attacked by another guest |
| AS-8 | Availability of the guest network | Guests have no Wi-Fi |
| AS-9 | Truth of what Enkrnett says | Operator or guests believe in protection that is not there |
| AS-10 | The device's software | The device does what an attacker wants |
| AS-11 | Guests' privacy | Guests are tracked |

## 3. Who attacks

| ID | Attacker | Position and ability |
|---|---|---|
| AC-1 | **Guest** | Connected to the guest network. Sends any traffic. |
| AC-2 | **Listener** | Within radio range. Records. Sends nothing. |
| AC-3 | **Nearby sender** | Within radio range. Sends frames; runs a fake network. |
| AC-4 | **Key holder** | Knows the management key, not the owner password. For example a former employee. |
| AC-5 | **Site device** | A device on the site network under an attacker's control. |
| AC-6 | **Web page** | A page open in the operator's browser. It acts through that browser. |
| AC-7 | **Internet host** | Anywhere on the Internet. |
| AC-8 | **Person at the device** | Can touch the device: power, buttons, ports. |
| AC-9 | **Operator's phone** | Malware on the phone the operator manages with. |
| AC-10 | **Supplier** | Can alter an image, a package, or an input of the build. |

## 4. Where attacks enter

| Entry | Offered to |
|---|---|
| Guest network: OWE BSS and, in Transition Mode, open BSS | Everyone in range |
| Management network | Holders of the management key |
| Setup network | Everyone in range, while the setup window is open |
| DHCP and DNS on the device | Clients of each Wi-Fi network |
| Management UI and API | Clients of the management and setup networks |
| Uplink | Nothing is offered. Outgoing connections only. |
| Power, buttons, Ethernet ports | A person at the device |
| The operator's browser | Pages the operator opens |

## 5. Threats

Verification "Test" means a test derived from the named requirement. "M-8" is the measurement
of RFC 006. "R-n" refers to section 7.

### 5.1 Becoming the owner, or acting as the owner

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 1 | Claims the device during the setup window before the operator does | AC-3 | AS-1 | ENK-OWN-001, 004, 005, 008 | Test | R-1 |
| 2 | Runs a fake setup network and relays the operator's setup | AC-3 | AS-1, AS-2, AS-3 | ENK-OWN-004, 006 | Test | R-2 |
| 3 | Overhears what the operator enters during setup | AC-2 | AS-2, AS-3 | OWE on the setup network (RFC 002 §1); **ENK-OWN-022** | Test | R-3 |
| 4 | Guesses the owner password over the network | AC-4 | AS-2 | ENK-OWN-009, 010, 012 | Test | R-4 |
| 5 | Guesses the management key from a recording | AC-2 | AS-3 | ENK-OWN-014, 015 | Inspection | None known |
| 6 | Reads the owner's session on the management network | AC-4 | AS-2 | WPA3-Personal where the phone supports it (RFC 002 §1) | Test | R-5 |
| 7 | Uses a session that belongs to the owner | AC-4, AC-9 | AS-1 | ENK-AUTH-002, 003, 005, 006 | Test | Until the session ends |
| 8 | Finds a secret in a log, a status, or a diagnosis | AC-4, AC-8 | AS-2, AS-3 | ENK-OWN-011; **ENK-SEC-016** | Search of every output | None known |
| 9 | Finds a secret in the browser's cache | AC-9 | AS-3 | **ENK-SEC-015** | Test | None known |

### 5.2 Acting through the operator's browser

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 10 | Sends requests to the device in the operator's name | AC-6 | AS-1, AS-4 | ENK-SEC-010, 011 | Test from another origin | None known |
| 11 | Reaches the device under a foreign name (DNS rebinding) | AC-6 | AS-1, AS-4 | ENK-SEC-009 | Test | None known |
| 12 | Shows the management UI inside another page to steer clicks | AC-6 | AS-4 | ENK-SEC-012 | Test | None known |
| 13 | Supplies a value that the management UI runs as script, for example a name that a guest's device announces | AC-1 | AS-1 | ENK-SEC-012; **ENK-SEC-013** | Test with hostile values | None known |

### 5.3 Reaching the management plane or the platform

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 14 | Reaches the management UI or API from the guest network | AC-1 | AS-1 | ENK-ISO-003, ENK-NET-009 | M-8; Test | None known |
| 15 | Reaches a platform service from the guest network | AC-1 | AS-10 | ENK-ISO-003; ENK-HARD-002, 004, 005 | M-8; Test | None known |
| 16 | Reaches the device from the site network | AC-5 | AS-1, AS-10 | ENK-ISO-005 | M-8; Test | None known |
| 17 | Reaches the device from the Internet | AC-7 | AS-1, AS-10 | ENK-ISO-005; ENK-TOPO-001 | Test | None known |
| 18 | Reaches the device over IPv6 | AC-1, AC-5 | AS-1, AS-10 | ENK-NET-008 | M-8; Test | None known |
| 19 | Joins a network through WPS | AC-3, AC-8 | AS-3 | **ENK-HARD-008** | Inspection; Test | None known |
| 20 | Obtains a shell through failsafe mode | AC-8 | AS-10 | ENK-HARD-003 | Test | None known |
| 21 | Plugs into a free Ethernet port | AC-8 | AS-6 | ENK-TOPO-004 | Test | None known |
| 22 | Makes a value from a request reach a command interpreter | AC-4, AC-3 | AS-10 | **ENK-SEC-014** | Inspection; Test with hostile values | None known |
| 23 | Exhausts the daemon with large, malformed, or slow requests | AC-4, AC-3 | AS-8 | Limits set by the API RFC | — | Until that RFC is accepted |

### 5.4 Guests against the site network and against each other

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 24 | Reaches a device on the site network | AC-1 | AS-6 | ENK-ISO-001, 006 | M-8; Test | None known |
| 25 | Reaches another guest through the device | AC-1 | AS-7 | ENK-ISO-002 | M-8; Test | None known |
| 26 | Sends broadcast frames to other guests, using the group key that all clients of one BSS share | AC-1 | AS-7 | None | M-8 | R-6 |
| 27 | Answers other guests' requests for an address or for the router | AC-1 | AS-7 | ENK-ISO-002 | M-8 | Through R-6 |
| 28 | Reaches the site network or a guest from the management network | AC-4 | AS-6, AS-7 | ENK-ISO-004 | M-8; Test | None known |

### 5.5 The radio link

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 29 | Listens to a guest's traffic | AC-2 | AS-5 | OWE | M-4 | R-7 |
| 30 | Runs a fake access point with the guest network's name | AC-3 | AS-5 | None possible | — | R-8 |
| 31 | Pushes a client that supports OWE onto the open network | AC-3 | AS-5 | PMF, which OWE requires; the OWE policy RFC | M-4 | R-9 |
| 32 | Disturbs the Wi-Fi by interference or by a flood of connection attempts | AC-3 | AS-8 | None | — | R-10 |

### 5.6 At the device

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 33 | Resets the device and claims it | AC-8 | AS-1, AS-4 | None. Reset by button is intended (ENK-OWN-017). | — | R-11 |
| 34 | Restarts the device or removes power | AC-8 | AS-8 | None | — | R-11 |
| 35 | Opens the case | AC-8 | All | Out of scope | — | — |

### 5.7 Software

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 36 | Uses a flaw in a platform component that answers on the guest network | AC-1 | AS-10 | ENK-HARD-005; ENK-BASE-002, 003 | Inspection | R-12 |
| 37 | Uses a flaw in the Enkrnett daemon | AC-4, AC-3 | AS-10 | Memory-safe language; ENK-SEC-014; a requirement for least privilege after measurement M-5 | Review | Until that requirement exists |
| 38 | Delivers an altered image, package, or update | AC-10 | AS-10 | The RFC on build, release, and update | — | Until that RFC is accepted |

### 5.8 Truth and privacy

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 39 | Enkrnett states more protection than exists | — | AS-9 | ENK-SCOPE-004; the OWE status RFC | Review of every status and message | Until that RFC is accepted |
| 40 | The UI shows a state from before an update | — | AS-9 | ENK-MGMT-005 | Test | None known |
| 41 | The device keeps records of guests | — | AS-11 | Decision D-15 | — | Until decided |
| 42 | The operator's phone is under an attacker's control | AC-9 | AS-1 | None beyond ENK-AUTH-003 | — | R-13 |

## 6. Requirements added by this RFC

| ID | Requirement | Against | Verification |
|---|---|---|---|
| ENK-OWN-022 | In state `SETUP`, before the operator enters any value, the management UI shall state whether the operator's phone is connected to the setup network with or without encryption. | 3 | Test with a client that supports OWE and one that does not |
| ENK-SEC-013 | The management UI shall show every value that does not come from its own code as text. It shall never interpret such a value as markup or script. | 13 | Test with hostile values in every field that is displayed |
| ENK-SEC-014 | The daemon shall pass no value that it received in a request to a command interpreter. | 22 | Inspection; test with hostile values |
| ENK-SEC-015 | A response that contains the management key or a session identifier shall forbid every cache to store it. | 9 | Inspection of response headers |
| ENK-SEC-016 | No log, status, or diagnosis shall contain the owner password, the management key, or a session identifier. | 8 | Search of every output after using every function |
| ENK-HARD-008 | WPS shall not be available on any network. | 19 | Inspection of the image; test |

## 7. Risks that remain

| ID | Risk | Who | Effect | How it is noticed or reduced | What would close it |
|---|---|---|---|---|---|
| R-1 | The device is claimed by someone else during the setup window. | AC-3 | The operator cannot finish setup. | Setup fails visibly. Reset and repeat. | A secret given to the device at installation |
| R-2 | A relayed fake setup network learns the owner password and the management key. | AC-3 | Full control, unnoticed. | Needs presence and preparation at the moment of setup. | Same, with a password-authenticated key exchange |
| R-3 | Setup from a phone without OWE can be overheard. | AC-2 | As R-2. | The UI warns before anything is entered (ENK-OWN-022). | Same |
| R-4 | Repeated wrong passwords keep the owner from logging in. | AC-4 | No management until the delay ends. | Reset ends it. Replacing the management key excludes the attacker. | — |
| R-5 | With WPA2, a key holder can decrypt another client's session, including the owner password. | AC-4 | Full control. | The key is private. Phones that use WPA3 are not affected. | Password-authenticated login; WPA3 only |
| R-6 | A guest can send broadcast frames to other guests. | AC-1 | Depends on the victim's device. | Stated to the operator. | Per-client group keys, where the platform offers them |
| R-7 | In Transition Mode, clients without OWE are not encrypted. | AC-2 | Those guests are overheard. | The UI shows how many. | OWE only |
| R-8 | A fake access point with the same name is accepted by guests' devices. | AC-3 | Those guests' traffic passes through the attacker. | Stated to operator and guests. | Not possible with OWE |
| R-9 | A client that supports OWE is pushed onto the open network. | AC-3 | That guest is overheard. | PMF stops forged disconnects. | OWE only; Transition Disable |
| R-10 | The Wi-Fi is disturbed. | AC-3 | No service while it lasts. | — | Not possible |
| R-11 | A person at the device restarts it, or resets and claims it. | AC-8 | No service; or a device under that person's control. | The operator's management key stops working. The guide advises placing the device out of reach. | Placement |
| R-12 | A flaw in a platform component is used before an update is installed. On targets with reduced platform hardening the effect is greater. | AC-1 | Up to full control. | Few services; updates. | — |
| R-13 | The operator's phone is under an attacker's control. | AC-9 | Full control while the owner is logged in. | Sessions end after 15 minutes without use. | — |

## 8. Keeping the model current

| Rule | |
|---|---|
| When | At every release, and whenever an RFC changes a data flow, an integration, or authorization |
| How | By an RFC that amends this one. Threat numbers are never reused. |
| Release checklist | States the RFC that last amended the model |

## Consequences

| Affected | Change |
|---|---|
| RFC 002 | Gains ENK-HARD-008 |
| RFC 003 | Gains ENK-SEC-013 to 016 |
| RFC 004 | Gains ENK-OWN-022. Its section 9 is restated by section 7 here. |
| Draft threat table and adversary classes | Replaced by sections 3, 5, and 7 |
| Draft requirement "no shell injection path" | Replaced by ENK-SEC-014 |
| Planned RFCs on the API, on OWE status, on configuration, on build and update | Each closes the threats that name it |

## Alternatives considered

| Alternative | Why not |
|---|---|
| A threat model after the first implementation | Tests could not be derived from it, and controls would be added to code that was not designed for them. |
| A general list of good practices | It cannot be tested and it names no remaining risk. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Limits for threat 23 | API RFC |
| 2 | Least privilege for threat 37 | Measurement M-5 |
| 3 | Threat 38 | RFC on build, release, update |
| 4 | Threat 41 | Decision D-15 |

## References

[rfc8110]: https://www.rfc-editor.org/rfc/rfc8110.txt
[airsnitch]: https://www.ndss-symposium.org/wp-content/uploads/2026-f1282-paper.pdf
[wicg-lna]: https://wicg.github.io/local-network-access/
[wfa-spec]: https://www.wi-fi.org/system/files/WPA3%20Specification%20v3.5.pdf

- RFC 8110, Opportunistic Wireless Encryption: [text][rfc8110]
- Zhou et al., "AirSnitch", NDSS 2026 (threats 25 to 27): [paper][airsnitch]
- WICG, Local Network Access (threats 10 and 11): [specification][wicg-lna]
- Wi-Fi Alliance, WPA3 Specification v3.5 (threat 31): [PDF][wfa-spec]
