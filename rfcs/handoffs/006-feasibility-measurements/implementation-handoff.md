# Procedures — RFC 006 Feasibility measurements

**Entry point:** [README.md](./README.md)

Each measurement has: question, setup, procedure, what to record. Where a name of a file, a
package, or an option is given, it was checked against OpenWrt 25.12.5. Where it is not given,
choosing it is part of the work; record the choice.

## Common facts

| Item | Value |
|---|---|
| OpenWrt release | 25.12.5 |
| Package manager | `apk` |
| Default Wi-Fi package | `wpad-basic-mbedtls` (access point and station; includes OWE) |
| Package architecture of MT7621 and MT7628 | `mipsel_24kc` |
| Wi-Fi option for OWE | `option encryption 'owe'` |
| Wi-Fi option for automatic Transition Mode | `option owe_transition '1'` |
| Wi-Fi options for isolation | `option isolate '1'`, `option bridge_isolate '1'` |
| Station list of one BSS | `ubus call hostapd.<ifname> get_clients` |
| State of one BSS | `ubus call hostapd.<ifname> get_status` |
| State of all radios | `ubus call network.wireless status` |
| Downloads | `https://downloads.openwrt.org/releases/25.12.5/targets/<target>/<subtarget>/` |

Known defect to watch for: in automatic Transition Mode both BSSes can receive the same BSSID,
and the second then fails to start. The fix is OpenWrt commit `1789118599` and is not part of
25.12.5.

---

## M-6 — Simulated radios in a virtual machine

**Question.** Does OpenWrt in a virtual machine with simulated radios run OWE from association
to traffic?

### Setup

| Item | Value |
|---|---|
| Image | `openwrt-25.12.5-x86-64-generic-ext4-combined.img.gz` from `targets/x86/64/` |
| Packages to add | `kmod-mac80211-hwsim`, `wpad-basic-mbedtls`, `iw`, `ip-full` |
| Radios | At least 3 simulated radios |
| Station | Runs in the same virtual machine, in its own network namespace, so that its traffic crosses the simulated radio |

### Procedure

| Step | Action |
|---|---|
| 1 | Start the virtual machine. Install the packages. Record `apk list --installed`. |
| 2 | Confirm that the simulated radios exist. |
| 3 | Configure one access point with OWE through UCI. Start it. |
| 4 | Connect a station with OWE and PMF required. Obtain an address. Exchange traffic with the access point. |
| 5 | Read the station list and the BSS state through ubus. |
| 6 | Change the access point to automatic Transition Mode. Start it. |
| 7 | Confirm that two BSSes run, with different BSSIDs, and that the hidden one carries the name `<ssid>OWE`. |
| 8 | Connect one station with OWE and one without. Note which BSS each joins. |
| 9 | Repeat steps 3 to 8 from a script, without manual action. |

### Record

| Item |
|---|
| Result of every step |
| Output of the ubus calls in steps 5 and 8 |
| Whether the defect named under "Common facts" appears |
| Time from start of the virtual machine to the end of step 8 |
| Whether step 9 runs unattended, and what prevents it if not |

---

## M-5 — Confirm-or-revert by a process that is not root

**Question.** Can configuration be applied with confirm-or-revert through OpenWrt's own
interfaces, by a process that is not `root`?

### Setup

The virtual machine of M-6, with a running access point and one connected station. Add `rpcd`.

### Procedure

| Step | Action |
|---|---|
| 1 | List the ubus objects and methods needed to: read the state of the radios and of each BSS; read the stations of each BSS; change the Wi-Fi, network, firewall, and DHCP configuration; make OpenWrt apply a changed configuration. |
| 2 | Create a user that is not `root`. Give it access to exactly the methods of step 1. Record the access-control files used. |
| 3 | As that user, carry out each item of step 1. |
| 4 | As that user, try three things outside the list of step 1. Each must be refused. |
| 5 | As that user, change the name of the access point using `uci apply` with `rollback` and a `timeout`. Do not confirm. Observe whether the previous name returns, and when. |
| 6 | Repeat step 5 and confirm within the timeout. Observe that the new name stays. |
| 7 | Repeat step 5 and restart the virtual machine inside the timeout. Observe which name is active afterwards. |
| 8 | Without `rpcd`: find out how a user that is not `root` can change only the configuration files it is meant to change. |
| 9 | For each of these changes, measure how long no station can be connected: name of the network; OWE to Transition Mode; key of a WPA2/WPA3 network. |

### Record

| Item |
|---|
| The list of step 1 |
| Result of steps 3 to 8, with the exact calls |
| Whether a session is needed for `uci apply`, and how a local process obtains one |
| The durations of step 9 |

---

## M-2 — Daemon footprint on `mipsel_24kc`

**Question.** How large is a daemon of Enkrnett's kind, and how much memory does it use?

### Setup

| Item | Value |
|---|---|
| Toolchain | `openwrt-sdk-25.12.5-ramips-mt7621_gcc-14.3.0_musl.Linux-x86_64.tar.zst`, with Rust from OpenWrt's packages feed |
| Build settings | Optimise for size; link-time optimisation; one code generation unit; abort on panic; symbols stripped |
| Run target | OpenWrt `malta/le` in QEMU, or a device |

Three throwaway programs:

| Program | Does |
|---|---|
| **A** floor | Listens on a TCP port and answers every request with one fixed HTTP response. Standard library only. |
| **B** lean | Blocking HTTP server; reads and writes one small JSON document; checks a password against a salted slow hash; produces random values; sends one request to ubus and reads the answer. No asynchronous runtime. |
| **C** asynchronous | The function of B on an asynchronous runtime with a common web framework. It shows what the budget excludes. |

### Procedure

| Step | Action |
|---|---|
| 1 | Build A, B, and C. Record the compiler version and every dependency with its version. |
| 2 | For each: measure the size of the executable. |
| 3 | For each: measure the size it occupies inside squashfs, using the compression settings OpenWrt uses for the target. |
| 4 | For each: start it on the run target, leave it idle for 5 minutes, and measure resident and proportional memory. |
| 5 | For B: send 1,000 requests and measure memory again. |
| 6 | For B, on a device with an MT7628 if one is at hand: measure the time of one password check for each hash function considered. State the parameters. |
| 7 | For B: try two ways of reaching ubus (own client on the socket; dynamic linking to the platform's library) and record size and effort for each. |

### Record

| Item |
|---|
| A table: program × (executable size, size in squashfs, idle memory, memory after load) |
| Dependency lists |
| Password check times, or the statement that no device was at hand |
| Anything that did not build for `mipsel_24kc`, with the error |

---

## M-1 — Phone browsers

**Question.** What do phone browsers do with a UI served over HTTP from a private address?

### Setup

| Item | Value |
|---|---|
| Server | The owner's workstation. Its Wi-Fi adapter offers one network at a time. It serves one static page with a form over HTTP at its own address. |
| Network W | WPA2/WPA3-Personal, with Internet |
| Network N | Open or OWE, without Internet. Transition Mode needs two networks at once and is not possible on this adapter; it is covered by M-4. |
| Name | The workstation's DNS service answers one fixed name with the workstation's own address |
| Clients | Google Pixel (Android) and iPad, in their default state |

### Preparation (development team)

| Item | Content |
|---|---|
| Test page | One static page: a form with a password field; a button that stores and shows a value in session storage; a button that copies a text; a line that shows the time the page was built |
| Networks | Exact steps to start W, to start N, and to stop each, on the workstation. W and N are used one after the other. |
| Caching | Two ways to serve the page: with headers that forbid caching, and without |
| Connectivity check | A way to answer the phone's connectivity check with a redirect to the page, for row 14 |
| Observation sheet | The 15 rows below as a sheet with a yes/no box and a line for notes, once per client |
| Restoring | Steps that return the workstation to its previous state |

Nothing of this is kept in the repository.

### What the iPad cannot show

| Subject | Reason | Consequence |
|---|---|---|
| Falling back to mobile data | An iPad without a mobile subscription has no mobile data | Rows 13 and 15 are complete for the Pixel only. The report says so. |
| The narrow screen of a phone | — | Not part of this measurement |

The browser engine is the same on iPad and iPhone. Record the iPad's model and system version:
OWE needs a model from late 2020 or later.

### Procedure

Carry out every row on each client. Answer yes or no, and describe what appeared.

| # | Observation |
|---|---|
| 1 | On W, typing `http://<address>/` opens the page. |
| 2 | On W, typing `http://<name>/` opens the page, and not a web search. |
| 3 | Row 2 with the phone's private DNS service switched on. |
| 4 | Scanning a QR code of the address with the camera application opens the page. |
| 5 | Scanning a Wi-Fi QR code joins W. The same for N. |
| 6 | What the browser shows about the connection not being HTTPS, and where. |
| 7 | Submitting the form works. |
| 8 | The browser offers to save a password typed into the form, and fills it in on the next visit. |
| 9 | A value in session storage survives a reload and is gone after the tab is closed. |
| 10 | A button can copy a text to the clipboard. |
| 11 | The page can be added to the home screen. What opens from there. Whether a saved value is still present there. |
| 12 | After the server's files changed, a reload shows the new page. Repeat with the page sent once with, and once without, headers that forbid caching. |
| 13 | On N: which question the phone asks, and after which answer the page opens. |
| 14 | On N: when the device answers the phone's connectivity check with a redirect to the page, whether the phone opens the page by itself, and what works in that window (form, script, session storage, copying). |
| 15 | During 10 minutes on N: whether the phone leaves the network by itself. |

### Record

| Item |
|---|
| Model, operating system version, browser version of each client |
| The table above per client |
| Screenshots of rows 6, 13, and 14 |

---

## M-4 — OWE and Transition Mode on candidate devices

**Question.** How do OWE and Transition Mode behave on the candidate devices with real clients?

### Setup

| Item | Value |
|---|---|
| Device | A candidate of RFC 005 §7 with the OpenWrt 25.12.5 image for that device |
| Networks | Guest and management network on separate bridges, as in RFC 002 |
| Clients | As many of these as are at hand: iPhone 11 or later; an older iPhone; two Android phones with Android 10 or later from different makers; a phone with Android 9 or older; Windows 11; macOS 13 or later; Linux with NetworkManager; one other Wi-Fi device |

### Procedure

| Step | Action |
|---|---|
| 1 | Guest network with OWE only. Connect every client. |
| 2 | Guest network with automatic Transition Mode. Confirm both BSSes run. Connect every client. |
| 3 | Add the management network on the same radio as the guest network. Confirm that all three BSSes run at once. On a device with two radios, confirm the guest network on both. |
| 4 | With `isolate` and `bridge_isolate` set on the guest network: from a station on the open BSS, try to reach a station on the OWE BSS, and the reverse. |
| 5 | Read the station list of each BSS through ubus while clients of both kinds are connected. |
| 6 | Restart the device's Wi-Fi. Observe whether each client returns by itself. |
| 7 | Change the guest network from Transition Mode to OWE only. Observe what each client does with its saved network. |
| 8 | Set the guest network's name to 29, 30, and 32 bytes of ASCII, and to 9 and 10 Japanese characters, each in Transition Mode. Observe whether both BSSes start. |

### Record

| Item |
|---|
| Per client: joined or not in step 1; which BSS in step 2; how the network is shown in the client's list (lock, label, whether the hidden name appears); time to connect |
| Result of steps 3 to 8 |
| Output of step 5 |
| Device log for every failure |

---

## M-8 — Separation

**Question.** Does the separation of RFC 002 hold against a client that tries to break it?

### Setup

| Item | Value |
|---|---|
| Device | Configured by hand to match RFC 002 §1 to §3 |
| Site network | The router on the uplink, with one test host on it |
| Clients | Two machines on the guest network, one on the management network. Tools for port scanning and for sending hand-made frames. |

### Procedure

The last column gives the result that RFC 002 requires.

| # | From | Attempt | Expected |
|---|---|---|---|
| 1 | Guest | Reach the test host on the site network | Fails |
| 2 | Guest | Open the site router's administration page | Fails |
| 3 | Guest | Scan every port of the device | Only DHCP and DNS answer |
| 4 | Guest | Ask the device's DNS for the management name | No address |
| 5 | Guest | Reach the other guest: same BSS | Fails |
| 6 | Guest | Reach the other guest: open BSS to OWE BSS | Fails |
| 7 | Guest | Reach the other guest: other radio | Fails |
| 8 | Guest | Send a packet to the device's hardware address with the other guest's IP address | Not delivered |
| 9 | Guest | Reach the device or the other guest over IPv6, including link-local | Fails |
| 10 | Guest | Send a broadcast frame | **Reaches the other guest.** This is a stated limit. Record what arrives. |
| 11 | Management | Open the management address | Works |
| 12 | Management | Reach the Internet | Works |
| 13 | Management | Reach the test host on the site network | Fails |
| 14 | Management | Reach a guest | Fails |
| 15 | Site network | Scan every port of the device's uplink address | Nothing answers |

### Record

| Item |
|---|
| The device's complete configuration |
| Result of every row, with the tool and command used |
| Every row whose result differs from the expectation, with a capture |

---

## M-3 — Complete image on a `constrained` device

**Question.** Does a complete image fit and run on a `constrained` device?

### Setup

| Item | Value |
|---|---|
| Builder | `openwrt-imagebuilder-25.12.5-ramips-mt76x8.Linux-x86_64.tar.zst` |
| Device | A `constrained` candidate of RFC 005 §7 |
| Packages | The default set without LuCI and without what RFC 002 §3 does not need; with program B of M-2 and a placeholder of 150 KB that does not compress further |
| Configuration | RFC 002: guest network in Transition Mode, management network, routing, firewall, DHCP |

### Procedure

| Step | Action |
|---|---|
| 1 | Build the image. Record the package list and the size. Compare with the stock image of the same device. |
| 2 | Install. After the first start, measure free space in the overlay. |
| 3 | Measure free and available memory: idle; with all networks running; with 1, 5, and 10 clients connected and transferring. |
| 4 | Measure throughput of one guest client to the Internet side. |
| 5 | Run for 24 hours with clients connected. Record every out-of-memory event and every restart of a service. |

### Record

| Item |
|---|
| Image size, stock image size, image limit |
| Free overlay |
| Memory in each state of step 3, and the increase per client |
| Throughput |
| Events of step 5 |

---

## M-7 — Installation by non-experts

**Question.** Can non-experts install a candidate device, and recover it, with a written guide
alone?

### Setup

| Item | Value |
|---|---|
| Devices | Candidates of RFC 005 §7 with an image for the vendor's firmware, in the state in which they are sold. Record the version of the vendor's firmware. |
| Image | The OpenWrt 25.12.5 image for the device stands in for an Enkrnett image |
| Guide | A draft per device, reviewed by the architect before the trial |
| Participants | Three or more people who fit the description of the operator in RFC 001 §3 |

### Procedure

| Step | Action |
|---|---|
| 1 | The participant receives the device, the guide, and nothing else. No help is given. |
| 2 | The participant installs. Note the time, every hesitation, every error, and every question. |
| 3 | Note whether a phone alone was enough, or a computer was needed. |
| 4 | The participant follows the guide's recovery section on a device prepared for it. |
| 5 | Ask the participant what was unclear. |

### Record

| Item |
|---|
| Per participant: finished without help or not; time; errors; questions |
| Per device: whether the vendor's firmware accepted the image; whether a phone alone was enough |
| Changes to the guide that the trial suggests |
| No personal data of participants beyond their occupation |
