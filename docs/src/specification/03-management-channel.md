# 3. Management channel

From RFC 003, as amended by RFC 007 (ENK-SEC-013 to 016). Request limits (ENK-SEC-017 to 022) are
in chapter 9.

The device serves the management UI as static files, over HTTP, on the management network and the
setup network. The UI calls the management API on the same device. A hosted site carries
documentation and a demonstration, and cannot manage a device. This direction is revisable
(owner decision D-05); the API does not depend on where the UI is served from.

## 3.1 Why not HTTPS, a hosted page, or an app

| Fact |
|---|
| A page loaded over HTTPS may not call `fetch()` against `http://` (mixed content). Chrome 142+ lifts this for the local network after a permission; Safari has not, and every iOS browser uses Safari's engine. |
| A first-time operator on a setup network without Internet cannot load a hosted page at all. |
| Service workers, camera access, and `crypto.subtle` need a secure context; a page at `http://192.168.x.x` is not one. |
| No certificate that a phone trusts can be issued for a local address without a service outside the venue, which ENK-SCOPE-002 excludes. |
| A web page cannot browse for devices; it can only open an address it already knows. |
| Safari deletes a site's stored data after seven days without a visit. |

## 3.2 Delivery

```text
 Operator's phone                      Device
 ┌────────────────┐   management    ┌──────────────────────────┐
 │ Browser        │    network      │ Enkrnett daemon          │
 │  management UI │ ◄─────────────► │   static UI files        │
 │                │      HTTP       │   management API         │
 └────────────────┘                 └──────────────────────────┘
```

{{#include requirements/mgmt.md}}

The address is the reliable path; a name is offered only if measurement M-1 shows it works on
phones in their default state (ENK-MGMT-008, open). QR codes are read by the phone's own camera
application.

## 3.3 Request security

{{#include requirements/sec.md}}

ENK-SEC-017 to 022 (limits) are stated in chapter 9 §6 with the API they bound.

## 3.4 What protects the channel

| Layer | Provides | Limit |
|---|---|---|
| Management network (WPA2-Personal, WPA3-Personal) | Encryption and integrity between phone and device | With WPA2, someone who knows the management key and records the phone joining can decrypt that session. WPA3 closes this. The UI tells the owner which one their phone used (chapter 10, `connection.security`). |
| Owner password and session (chapter 4) | Authority to change anything | — |
| ENK-SEC-009 to 012 | Defence against pages on other origins | — |

## 3.5 What the operator sees

| Situation | Content (wording: chapter 10) |
|---|---|
| The browser shows "Not secure" | The page is delivered inside the management Wi-Fi without a web certificate; the management Wi-Fi encrypts it; keep its password private. |
| The page is opened on the guest network | Nothing answers (ENK-ISO-003). The guide says to join the management Wi-Fi first. |
| The phone asks whether to stay on a Wi-Fi network without Internet | The guide says which answer keeps the phone connected (M-1, open). |

## 3.6 Hosted site

| Content | Rule |
|---|---|
| Documentation | — |
| Demonstration of the UI against simulated data | Every page states that it is a demonstration and is connected to no device. It offers no way to enter a device address. |
| Downloads | Chapter 5 and chapter 11 |

## 3.7 When to revisit

| Trigger | Then |
|---|---|
| Safari ships local-network permission with the mixed-content exemption | A hosted client becomes possible on all phones; consider adding it beside the device-served UI. |
| A browser warns on, or blocks, plain HTTP to private addresses | Reassess before the change reaches stable. |
| A standard for certificates on local devices is adopted by browsers | Consider TLS on the standard device class. |
