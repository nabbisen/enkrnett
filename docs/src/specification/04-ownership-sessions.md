# 4. Ownership and sessions

From RFC 004, as amended by RFC 007 (ENK-OWN-022), RFC 009 (Erase), RFC 015 (greeting; how a key
replacement becomes final), and RFC 016 (step 5; 600 s; the label address; *settings*).

An unclaimed device opens a **setup window** when it is powered on. Whoever completes setup in
that window becomes the **owner** and chooses the **owner password**. Every change afterwards
needs that password. There is one owner and no other role. A lost password or key is recovered
by **Erase**, which removes everything.

## 4.1 States

```text
              power on            operator confirms        owner logs in
  UNCLAIMED ───────────►  SETUP  ──────────────────►  PENDING  ─────────────►  CLAIMED
      ▲                     │  ▲                         │                        │
      │    window closes    │  │   10 minutes pass,      │                        │
      └─────────────────────┘  │   or the device restarts│                        │
                               └─────────────────────────┘                        │
      ▲                                                                           │
      └────────────────────────────────── Erase ──────────────────────────────────┘
```

| State | Meaning | Networks offered |
|---|---|---|
| `UNCLAIMED` | No owner. Setup window closed. | None |
| `SETUP` | No owner. Setup window open. | Setup network |
| `PENDING` | Setup was confirmed. The claim is not final yet. | Management network |
| `CLAIMED` | The device has an owner. | Management network; guest network according to the settings |

| From | Event | To |
|---|---|---|
| `UNCLAIMED` | The device finishes starting | `SETUP` |
| `SETUP` | The setup window closes | `UNCLAIMED` |
| `SETUP` | The operator confirms setup | `PENDING` |
| `PENDING` | The owner logs in through the management network | `CLAIMED` |
| `PENDING` | 10 minutes pass without that login, or the device restarts | `SETUP` |
| `CLAIMED` | The device restarts | `CLAIMED` |
| `CLAIMED` | Erase | `UNCLAIMED`, then the device restarts |

These are the only states and transitions. A device that restarts without an owner starts in
`UNCLAIMED` and opens the setup window.

## 4.2 Setup window and claim

Opening the window needs a physical act: connecting power. A return from `PENDING` continues the
window that was open.

| Step | Who | What happens | Screen (chapter 10) |
|---|---|---|---|
| 1 | Operator | Joins the setup network whose name matches the label. | — |
| 2 | Operator | Opens the management UI. The UI shows whether the phone's connection is encrypted (ENK-OWN-022). | S-1 |
| 3 | Device | Grants this client the claim session. Any other client is told that setup is in progress elsewhere. | S-1 |
| 4 | Operator | Asks the device to identify itself. The device's light blinks. The operator confirms seeing it. | S-2 |
| 5 | Operator | Chooses the country, the guest network's name and mode, and the owner password. | S-3, S-4, S-5 |
| 6 | Device | Checks every value. Shows the management network's name, the management key, and the address of the management UI. Nothing persists. | S-6 |
| 7 | Operator | Keeps them, then confirms. | S-6 |
| 8 | Device | State `PENDING`. Ends the setup network. Starts the management network. | S-7 |
| 9 | Operator | Joins the management network and logs in with the owner password. | S-8 |
| 10 | Device | State `CLAIMED`. The claim is **final**. Starts the guest network. | S-9 |

{{#include requirements/own.md}}

ENK-OWN-008 protects the operator who did not manage to keep the management key: a confirmed but
unreachable device returns to `SETUP` by itself. The same act, joining the new management network
within 600 s, settles a later change of the management network's name or key (chapter 6 §4,
ENK-API-021 and 022).

## 4.3 Sessions

{{#include requirements/auth.md}}

## 4.4 Authorization

There is one role: the owner.

| Resource | No session | Claim session | Owner session |
|---|---|---|---|
| Files of the management UI | ✓ | ✓ | ✓ |
| Greeting: `product`, `api`, `state`, `languages`, `development`, `connection` | ✓ | ✓ | ✓ |
| Claim (`claim/*`) | ✗ | ✓ | ✗ |
| Everything else | ✗ | ✗ | ✓ |

The greeting's `connection.security` tells the client how its own connection is protected
(`none`, `owe`, `wpa2`, `wpa3`, or `null` when the device cannot tell). The API matrix per state
is chapter 9 §2.

## 4.5 Erase

| Situation | Recovery |
|---|---|
| Owner password forgotten | Erase, then set up again |
| Management key lost | Erase, then set up again |
| Both lost | Erase, then set up again |

One recovery path for every case. OpenWrt's default button behaviour: a press shorter than
1 second restarts the device; holding for 5 seconds or longer erases it. A guide never says
"press reset"; it describes the act and the sign: "Hold the small button on the back for five
seconds. The light starts blinking. Let go."

## 4.6 Limits that remain

Enkrnett states these to the operator where they matter (ENK-SCOPE-004). The full list with
attackers is chapter 12.

| # | Limit | Noticed or reduced by |
|---|---|---|
| 1 | Someone else claims the device during the setup window. | The operator cannot finish setup. Erase and repeat. |
| 2 | A fake setup network relays the operator's setup. | The name matches the label (ENK-OWN-004); the identify step defeats a non-relaying attacker. |
| 3 | On a setup network joined without OWE, what the operator enters can be overheard. | The UI says so before anything is entered (ENK-OWN-022). |
| 4 | With WPA2, someone who knows the management key can decrypt another client's session. | The key is private; the UI says when the phone used WPA2; phones that use WPA3 are not affected. |
| 5 | Anyone who can touch the device can restart or erase it. | After an erase the guest network disappears. The guide advises placing the device out of reach. |
| 6 | Guessing the owner password. | ENK-OWN-012. |

Limits 2, 3, and 4 can be closed by a secret that reaches the operator outside the network,
combined with a password-authenticated key exchange: a later RFC.
