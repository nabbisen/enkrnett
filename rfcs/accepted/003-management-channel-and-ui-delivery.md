# RFC 003 — Management channel and UI delivery

**Status.** Accepted (2026-09-29)
**Tracks.** Where the management UI comes from and how it reaches the management API. Startup
decision D-05.
**Touches.** The daemon (HTTP service); the UI build; the hosted project site; operator guides.

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md). Networks are defined in
[RFC 002](./002-network-topology-and-separation.md).

## Summary

The device serves the management UI as static files, over HTTP, on the management network and the
setup network. The UI calls the management API on the same device. A hosted site carries
documentation and a demonstration, and cannot manage a device.

**This direction is revisable** (owner, D-05). The API does not depend on where the UI is served
from, so a hosted or native client can be added later without redesign.

## Motivation

The drafts planned a PWA hosted on an HTTPS site that calls the device over local HTTP. Browsers
do not allow this everywhere.

| Fact | Source |
|---|---|
| A page loaded over HTTPS may not call `fetch()` against `http://`. It is blocked as mixed content. | [MDN][mdn-mixed] |
| Chrome 142 and later lifts this for the local network after the user grants a permission. | [Google][google-https], [Chrome][chrome-lna] |
| Safari has not shipped this. Every iOS browser uses Safari's engine in practice. | [WebKit position][webkit-lna] |
| A first-time operator on a setup network without Internet cannot load a hosted page at all. | — |
| Service workers, camera access, and `crypto.subtle` need a secure context. A page at `http://192.168.x.x` is not one. | [MDN][mdn-secure] |
| A web page cannot browse for devices. It can only open an address it already knows. | [W3C][w3c-discovery] (discontinued) |
| Android resolves no `.local` name over mobile data, and keeps mobile data as the default network on Wi-Fi without Internet. | [Android][android-dns], [Android][android-default] |
| Safari deletes a site's stored data after seven days of browser use without a visit. | [WebKit][webkit-itp] |
| The authors of the local-network specification state that a router's administration interface "must be designed and implemented to defend against CSRF on its own". | [WICG][wicg-lna] §5.3 |
| Chrome 154 asks before opening plain-HTTP public sites. Private addresses are exempt. | [Google][google-https] |

## Design

### 1. Delivery

```text
 Operator's phone                      Device
 ┌────────────────┐   management    ┌──────────────────────────┐
 │ Browser        │    network      │ Enkrnett daemon          │
 │  management UI │ ◄─────────────► │   static UI files        │
 │                │      HTTP       │   management API         │
 └────────────────┘                 └──────────────────────────┘
```

| ID | Requirement | Verification |
|---|---|---|
| ENK-MGMT-001 | The device shall serve the management UI as static files. | Inspection |
| ENK-MGMT-002 | The management UI shall load nothing from outside the device. | Network capture while using every page |
| ENK-MGMT-003 | The management UI shall work completely without service workers, camera access, `crypto.subtle`, the clipboard API, persistent storage, and Web Bluetooth. | Test on a page served over HTTP |
| ENK-MGMT-004 | The management UI and the daemon shall be released as one unit with one version number, shown in the UI. | Inspection |
| ENK-MGMT-005 | After an update, the UI shown to the operator shall be the one that belongs to the running daemon. A UI from before the update shall not be used. | Test: update, reopen |
| ENK-MGMT-006 | The management API shall not depend on the client being the UI that the device serves. | Test with a separate client |

### 2. Address

| ID | Requirement | Verification |
|---|---|---|
| ENK-MGMT-007 | The management UI shall be reachable at one IPv4 address, the same on the management and the setup network. The address shall change only when ENK-TOPO-003 forces the subnet to change. | Test |
| ENK-MGMT-008 | The device should also answer under one fixed name on those networks. Whether it does is decided by measurement M-1. | RFC 006 |
| ENK-MGMT-009 | The address shall be given to the operator as text and as a QR code, in the install guide and in the management UI. | Inspection |

The address is the reliable path. A name can fail when the phone uses its own DNS service instead
of the network's. If M-1 shows that the name fails on phones in their default state, the name is
not offered at all, because something that works only sometimes confuses.

QR codes are read by the phone's own camera application. The UI does not need camera access.

### 3. Request security

| ID | Requirement | Verification |
|---|---|---|
| ENK-SEC-009 | The device shall refuse every request whose `Host` header is not its own address or name. | Test, including a DNS rebinding attempt |
| ENK-SEC-010 | The management API shall accept no credential that a browser attaches by itself. It shall use neither cookies nor HTTP authentication. | Inspection; cross-site request test |
| ENK-SEC-011 | The management API shall send no header that permits cross-origin access, and shall refuse preflight requests. | Test from a page on another origin |
| ENK-SEC-012 | Every response shall forbid framing, forbid content-type guessing, send no referrer, and allow scripts, styles, images, and connections from the device only. | Inspection of response headers |

With ENK-SEC-010 a page on another origin cannot act in the operator's name, because it has
nothing the API would accept. With ENK-SEC-009 a page cannot reach the device under a foreign
name. These hold wherever the UI is served from.

### 4. What protects the channel

| Layer | Provides | Limit |
|---|---|---|
| Management network (WPA2-Personal, WPA3-Personal) | Encryption and integrity between phone and device | With WPA2, someone who knows the management key and records the phone joining can decrypt that session. WPA3 closes this. |
| Owner password and session (RFC 004) | Authority to change anything | — |
| ENK-SEC-009 to 012 | Defence against pages on other origins | — |

The device has no TLS. No certificate that a phone trusts can be issued for a local address
without a service outside the venue, which ENK-SCOPE-002 excludes.

### 5. What the operator sees

| Situation | What the UI and the guide say |
|---|---|
| The browser shows "Not secure" | "This page is delivered inside your management Wi-Fi without a web certificate. Your management Wi-Fi encrypts it. Keep the management key private." |
| The page is opened on the guest network | Nothing answers (ENK-ISO-003). The guide says: "Join your management Wi-Fi first." |
| The phone asks whether to stay on a Wi-Fi network without Internet | The guide says which answer keeps the phone connected. |

The exact wording belongs to the UI RFC. The content is fixed here because it must be accurate
(ENK-SCOPE-004).

### 6. Hosted site

| Content | Rule |
|---|---|
| Documentation | — |
| Demonstration of the UI against simulated data | Every page states that it is a demonstration and is connected to no device. It offers no way to enter a device address. |
| Downloads | Subject to RFC 005 |

## Consequences

| Affected | Change |
|---|---|
| Draft requirement "static deployment" on a hosting platform | Applies to the hosted site only. |
| Draft requirement "browser-native capabilities" (QR scanning, mDNS, service worker) | Withdrawn. See ENK-MGMT-003 and ENK-MGMT-009. |
| Draft requirement "changes in the PWA shall not require firmware rebuilds" | Withdrawn. UI and daemon are one unit (ENK-MGMT-004). |
| Draft statement "the router stores no rich UI assets" | Withdrawn. The UI bundle has a size budget (decision D-10). |
| Draft baseline "mDNS discovery" | Withdrawn. |
| Version skew between UI and API | Cannot occur for the UI the device serves. |
| Startup constraint "PWA as the primary experience" | Relaxed by the owner. Installability becomes an enhancement for a later delivery. |

## Alternatives considered

| Alternative | Why not now |
|---|---|
| Hosted HTTPS PWA calling the device over HTTP | Blocked on every iOS browser. Needs Internet for the first load. The hosting origin would hold every operator's credential. |
| The same, relying on Chrome's permission | Excludes iOS. |
| Device with a publicly trusted certificate | Needs a service outside the venue to issue and renew it. |
| Hosted PWA reaching the device through WebRTC or WebTransport with a pinned certificate | Needs a DTLS or QUIC stack on the device, which measures megabytes on MIPS. WebTransport certificates expire within two weeks; the device has no clock battery. |
| Native application | App-store accounts, review, and a second code base. |
| Self-signed certificate accepted by the operator | Browsers show a warning that a non-expert cannot judge. |

## When to revisit

| Trigger | Then |
|---|---|
| Safari ships local-network permission with the mixed-content exemption in a stable release | A hosted client becomes possible on all phones. Consider adding it beside the device-served UI. |
| A browser starts to warn on, or block, plain HTTP to private addresses | Reassess the delivery before the change reaches stable. |
| A standard for certificates on local devices is adopted by browsers | Consider TLS on the standard device class. |

## Verification

| Item | Where |
|---|---|
| Behavior of iOS Safari and Android Chrome with a page served over HTTP from a private address: loading, forms, session storage, saving the password, adding to the home screen, warnings | RFC 006, M-1 |
| Name resolution on phones in their default state | RFC 006, M-1 |
| Behavior on a Wi-Fi network without Internet | RFC 006, M-1 |

## Open questions

| # | Question |
|---|---|
| 1 | The fixed name, if M-1 supports offering one. Proposal: a name under `home.arpa`, which is reserved for local use ([RFC 8375][rfc8375]). |
| 2 | Request limits (sizes, rates, timeouts) are set by the API RFC. |

## References

[mdn-mixed]: https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Mixed_content
[mdn-secure]: https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Secure_Contexts/features_restricted_to_secure_contexts
[google-https]: https://blog.google/security/https-by-defau/
[chrome-lna]: https://developer.chrome.com/blog/local-network-access
[webkit-lna]: https://github.com/WebKit/standards-positions/issues/520
[webkit-itp]: https://webkit.org/tracking-prevention/
[w3c-discovery]: https://www.w3.org/TR/discovery-api/
[android-dns]: https://source.android.com/docs/core/ota/modular-system/dns-resolver
[android-default]: https://developer.android.com/develop/connectivity/network-ops/reading-network-state
[wicg-lna]: https://wicg.github.io/local-network-access/
[rfc8375]: https://www.rfc-editor.org/rfc/rfc8375.txt

- MDN, "Mixed content": [link][mdn-mixed]
- MDN, "Features restricted to secure contexts": [link][mdn-secure]
- Google, "HTTPS by default" (2025-10-28): [link][google-https]
- Chrome for Developers, "Local network access": [link][chrome-lna]
- WebKit standards position on local network access: [link][webkit-lna]
- WebKit, "Tracking prevention": [link][webkit-itp]
- Android, "DNS resolver": [link][android-dns]
- WICG, "Local Network Access": [link][wicg-lna]
