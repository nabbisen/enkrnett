#!/usr/bin/env python3
"""Requirement-to-test traceability (chapter 14 rule 1).

Every test names the requirement identifiers it verifies in its file header or function name.
This script lists, for every requirement of the specification, the test files that name it, and
writes the table to docs/src/verification/traceability.md. It exits non-zero when a requirement
allocated to the release given with --release has no test and no inspection note.

    tests/traceability.py            write the table
    tests/traceability.py --release 0.2.0   also fail on uncovered 0.2.0 requirements
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQ_DIR = ROOT / "docs/src/specification/requirements"
TEST_GLOBS = [
    "crates/*/src/**/*.rs",
    "crates/*/tests/**/*.rs",
    "ui/src/**/*.test.ts",
    "ui/scripts/*.mjs",
    "tests/browser/tests/*.ts",
    "tests/vm/*.sh",
    "openwrt/*.sh",
    "ui/scripts/*.mjs",
]
# Requirements verified by inspection, review, or a measurement rather than a test, with where.
INSPECTION = {
    "ENK-SCOPE-001": "crates/enkrnettd/src/platform/openwrt.rs: hostapd reached only through UCI and ubus",
    "ENK-SCOPE-002": "tests/vm: the claim on a VM whose uplink is slirp; no outside service is called",
    "ENK-SCOPE-006": "rfcs/README.md, the Amended-by lines",
    "ENK-MGMT-003": "tests/browser/tests/resilience.spec.ts (storage blocked); no service worker, camera, clipboard, or crypto.subtle in ui/src",
    "ENK-MGMT-008": "open: measurement M-1",
    "ENK-SEC-013": "ui: every API value rendered as text (Svelte text nodes; {@html} only for the QR SVG of our own making)",
    "ENK-SEC-014": "crates/enkrnettd: no shell and no std::process::Command; the one program run, /bin/uclient-fetch, is started with posix_spawn and an argv built from the configured URL, nothing from a request (platform/spawn.rs); crates/enkrnett-ubus speaks ubus directly",
    "ENK-SEC-015": "crates/enkrnettd/src/http/response.rs: no-store on every API response",
    "ENK-SEC-016": "crates/enkrnettd/src/log.rs and app.rs: events carry no key, password, or token",
    "ENK-API-027": "crates/enkrnettd/src/app/readings.rs: readings at most 2 s old; tests/effective.rs",
    "ENK-CFG-002": "crates/enkrnettd/src/platform/openwrt.rs: hostapd is reached through UCI (wireless) and ubus (hostapd.*) only; no file under /var/run/hostapd, no control socket",
    "ENK-CFG-004": "openwrt/patches/0001-wifi-scripts-owe-transition-bss-mac.patch carried; still needed on 25.12.5 (release 0.2.0 checked the header); openwrt/build-image.sh applies it to the locked wifi-scripts file in every release image; the test VM of tests/vm/image.sh applies it at first boot (tests/vm/files)",
    "ENK-GUEST-002": "release 0.2.0: the notice (ui notice.*), the status counts and the connection line reviewed against chapter 12 §7; docs/src/releases/0.2.0.md",
    "ENK-NET-009": "no management name is published in 0.2.0 (chapter 3: the address is the path, M-1 open); the guest dnsmasq instance carries no address entry (plan.rs dnsmasq(.., None)); tests/vm/isolation.sh: a guest gets DNS and DHCP from the device and nothing else",
    "ENK-OWE-007": "ui/src/lib/i18n/*.json mode.open, mode.encrypted; enkrnett_model::settings::GuestMode has the two values",
    "ENK-OWE-009": "plan.rs: the OWE BSS uses encryption owe, for which wifi-scripts sets ieee80211w=2 (M-6); open: hostapd get_status exposes no ieee80211w, so the daemon cannot read PMF back (recorded for the amendment RFC); the station side is visible as mfp in get_clients",
    "ENK-OWE-010": "plan.rs: no transition_disable option is written; tests/vm/isolation.sh reads the generated hostapd configuration",
    "ENK-OWE-012": "ui/src/lib/i18n/*.json settings.mode_note; ui/src/screens/Settings.svelte shows it on the switch to open_to_all",
    "ENK-OWE-015": "ui wording: status.counts.* say connected (associated), status.running says running; nothing claims what is on the air",
    "ENK-OWE-017": "ui/src/app.css: colour on .dot.running only; no badge or word sums protection up (tests/browser claimed.spec.ts ENK-UI-021)",
    "ENK-OWE-019": "open: ui compat.text names the generations from public sources; to be confirmed from M-4 (owner's kit) before the first release",
    "ENK-SCOPE-004": "release 0.2.0 review of every status and message against chapters 2, 7 and 12: docs/src/releases/0.2.0.md",
    "ENK-TOPO-002": "the VM takes its uplink by DHCP from QEMU's slirp (tests/vm); the three consumer routers are a device test (M3)",
    "ENK-TOPO-004": "openwrt/profiles/*/profile.json uplink_port; the platform leaves every other port without a network (prepare.sh: lan proto none models the image); per-device test open",
    "ENK-TOPO-005": "not implemented in 0.2.0 (should); device test when the Tier 1 hardware is in hand",
    "ENK-OWN-022": "tests/browser/tests/setup.spec.ts (the connection line on S-1); ui/src/components/ConnectionLine.svelte",
    "ENK-UI-018": "ui/src/lib/countdown.svelte.ts (performance.now); tests/browser setup.spec.ts shows the countdown",
    "ENK-UI-022": "ui components render API values as text; the only {@html} is the QR SVG; hostile-value test to add to the harness",
    "ENK-FOOT-001": "CI's mksquashfs figure (openwrt/build-mips.sh in the mips job): 300 KiB of 300 at M3 stage 3, the budget met with at most 4 KiB to spare; the workstation's xz estimate (286) runs about 14 KiB under it; docs/src/verification/m3.md stage 3",
    "ENK-FOOT-011": "browser harness: timing check to add; the bundle is 44 KiB compressed",
    "ENK-RF-003": "openwrt/profiles: every product profile fixes the country, so no radio transmits before it is set",
    "ENK-REPO-002": "openwrt/patches/0001-…: header with upstream reference and removal condition",
    "ENK-UPD-001": "crates/enkrnettd/src/app/update.rs: the daily check (a day by the wall clock, kept on flash) and update/check fetch the index URL and the same URL plus .sig through /bin/uclient-fetch over HTTPS and nothing else, a fixed GET per image with no identifier (tests/update.rs lists the fetches); tests/vm/update-check.sh on the real platform; the URL in /etc/config/enkrnett",
    "ENK-UPD-008": "the switch rides on update/check as {enable_checks} (deviation for the amendment RFC: the accepted API has no resource for it); ui/src/screens/Details.svelte offers it",
    "ENK-UPD-002": "ui/src/screens/Details.svelte: the update's own step with the version, the minutes, the Wi-Fi stops, cannot be undone from here, the baseline sentence; tests/browser/tests/update.spec.ts reads it; the daemon starts nothing without update/install from an owner session (crates/enkrnettd/tests/install.rs)",
    "ENK-UPD-006": "update.can_undo is false (crates/enkrnettd/src/app.rs update_shape); the step says 'This cannot be undone from here.' (tests/browser/tests/update.spec.ts)",
    "ENK-UPD-007": "one word, Update, in both languages (ui/src/lib/i18n, the word check); the baseline sentence shown when changes_baseline (ui/src/screens/Details.svelte); the M-7 reading is open",
    "ENK-RLS-001": "Cargo workspace version; tags 0.1.0 and 0.2.0 carry no prefix; enkrnett_model::release::Version parses major.minor.patch strictly",
    "ENK-RLS-002": "openwrt/build-image.sh names every image enkrnett-VERSION-BASELINE-PROFILE-KIND.bin (sysupgrade and the profile's vendor kind); enkrnett_model::release refuses any other name; crates/enkrnett-release index takes the name from the image file",
    "ENK-RLS-003": "ui/src/screens/Details.svelte version line; tests/browser claimed.spec.ts and setup.spec.ts match 'Enkrnett x.y.z on OpenWrt 25.12'",
    "ENK-RLS-004": "crates/enkrnett-release sign/verify (signify format); the daemon verifies index.json.sig before it reads the index (app/update.rs) and IMAGE.sig before the hand-over (app/update.rs, tests/install.rs); the root helper verifies IMAGE.sig again with usign; docs/src/contributors/releasing.md §4 signs every image and the index with the same key",
    "ENK-RLS-005": "openwrt/release-keys/*.pub go into /usr/share/enkrnett/keys (none yet); tests/update.rs: a signature by the designated next key verifies; the rotation procedure is docs/src/contributors/releasing.md §4; open: the rotation in CI waits for the first key",
    "ENK-RLS-006": "crates/enkrnett-model/src/release.rs (format, limits, the offer); crates/enkrnett-release index writes it; docs/src/design/release.md",
    "ENK-RLS-007": "crates/enkrnett-release keygen and sign run on the signing host; .github/workflows hold no key; openwrt/release-keys/README.md",
    "ENK-MGMT-001": "crates/enkrnettd/src/static_files.rs",
    "ENK-MGMT-004": "Cargo workspace version; ui/package.json",
    "ENK-MGMT-006": "tests/vm/contract.sh drives the API with curl",
    "ENK-MGMT-009": "ui/src/screens/Setup.svelte (S-6), Management.svelte (S-12)",
    "ENK-AUTH-002": "crates/enkrnettd/src/random.rs (32 bytes, base64url)",
    "ENK-AUTH-006": "crates/enkrnettd/src/session.rs (memory only)",
    "ENK-AUTH-008": "crates/enkrnettd/src/app.rs: every operation checks state, then session",
    "ENK-OWN-011": "crates/enkrnettd/src/auth.rs (argon2id); the hash is never returned",
    "ENK-OWN-014": "crates/enkrnettd/src/random.rs",
    "ENK-OWN-019": "crates/enkrnettd/src/app.rs: erase is the only path to UNCLAIMED",
    "ENK-OWN-021": "openwrt/package/enkrnett/files/enkrnett.init: no button handling",
    "ENK-API-007": "ui catalogues hold every operator sentence; the API sends codes",
    "ENK-API-009": "crates/enkrnett-model/src/api.rs: one struct per shape",
    "ENK-API-013": "crates/enkrnettd/src/api.rs ROUTES",
    "ENK-API-023": "crates/enkrnettd/src/app.rs management_key_post",
    "ENK-API-028": "crates/enkrnettd/src/log.rs (stderr to logd)",
    "ENK-API-029": "crates/enkrnettd/src/app.rs log::event calls",
    "ENK-API-031": "docs/src/design/daemon.md §7; ui/src/lib/errors.ts unknown values",
    "ENK-API-032": "crates/enkrnettd/src/api.rs PREFIX",
    "ENK-SEC-021": "crates/enkrnettd/src/api.rs: no upload resource",
    "ENK-WORD-002": "ui/src/lib/i18n/*.json erase.* entries",
    "ENK-WORD-003": "ui/src/screens/Main.svelte: Erase under Settings, Restart under Details",
    "ENK-UI-011": "release checklist (native-reader review)",
    "ENK-UI-012": "measurement M-7",
    "ENK-UI-014": "ui/src/screens/Setup.svelte: identify and country steps conditional",
    "ENK-UI-016": "ui/src/screens/Protects.svelte, app.css @media print",
    "ENK-UI-021": "ui/src/app.css: colour on .dot.running only",
    "ENK-UI-024": "ui/scripts/size.mjs; tests/browser claimed.spec.ts (no external request)",
    "ENK-UI-027": "ui/src/components/Header.svelte",
    "ENK-FOOT-012": "crates/enkrnettd/Cargo.toml: no async runtime, no framework",
    "ENK-FOOT-013": "crates/enkrnettd/Cargo.toml: no TLS crate",
    "ENK-FOOT-014": "crates/enkrnett-ubus",
    "ENK-FOOT-015": "openwrt/lock/inputs.json rustflags (-crt-static)",
    "ENK-FOOT-016": "docs/src/design/daemon.md §7",
    "ENK-FOOT-017": "openwrt/profiles/*/profile.json argon2",
    "ENK-CFG-019": "openwrt/package/enkrnett/files/enkrnett.init (user enkrnett)",
    "ENK-CFG-020": "openwrt/package/enkrnett/files/enkrnett.acl.json",
    "ENK-CFG-021": "measurement M-5b",
    "ENK-CFG-022": "openwrt/package/enkrnett/Makefile DEPENDS",
    "ENK-TIER-001": "openwrt/profiles/",
    "ENK-TIER-003": "ui/src/screens/Setup.svelte (identify, country); claim shape",
    "ENK-RF-001": "crates/enkrnettd/src/app.rs claim_check: the country before confirm",
    "ENK-RF-002": "openwrt/profiles/*/profile.json country.fixed",
    "ENK-RF-004": "chapter 6 §2: no such setting exists",
    "ENK-HARD-007": "ui/src/components/Header.svelte; --development",
    "ENK-REPO-001": "repository layout",
    "ENK-INST-001": "docs/src/guides/install-elecom-wrc-1167gst2.md and install-buffalo-wcr-1166ds.md: one guide per Tier 1 candidate, one file each (openwrt/build-image.sh, the factory kind); in English only by the owner's decision of 2026-11-27, the Japanese guides a controlled future plan (ROADMAP) and a deviation for the amendment RFC",
    "ENK-INST-002": "the install guides' first step is the label (model and the four characters) before anything is changed; open: observed in the usability test M-7 session S1 (rfcs/handoffs/006-feasibility-measurements/m-7-usability-test-plan.md)",
    "ENK-INST-003": "the install guides state the tested vendor firmware versions: none yet, honestly; filled from M-7 fact F-3 and openwrt/profiles/*/profile.json install.tested_vendor_firmware",
    "ENK-INST-004": "docs/src/guides/recover.md, 'An installation that was interrupted': steps 1 to 3 from the device's behaviour, step 4 the vendor's recovery marked to be confirmed; open: M-7 session S7",
    "ENK-BASE-001": "openwrt/lock/inputs.json openwrt.baseline 25.12 and release 25.12.5, the one baseline every image is built on; docs/src/releases/0.3.0.md",
    "ENK-BASE-002": "release checklist item 6 (docs/src/contributors/releasing.md §5): the baseline's end of life at least six months away at publication; nothing is published yet",
    "ENK-BASE-003": "update defined and tested: chapter 11 §2, tests/vm/update-install.sh (an image updated to the next on the VM); the upgrade to the next baseline is the same path with from_baseline set (enkrnett_model::release, tests/update.rs the upgrade offer); open: the first real upgrade across baselines",
    "ENK-BASE-004": "crates/enkrnettd/tests/install.rs (the successor's first boot keeps the owner, the key and the settings) and tests/vm/update-install.sh (ENK-UPD-004 rows: identifier, settings, owner's password kept; sessions gone)",
    "ENK-BASE-006": "release checklist (docs/src/contributors/releasing.md): every release ships an image for every Tier 1 profile from the lock (ci.yml job images) and the index offers the next release to the installed one; nothing is published yet",
    "ENK-BASE-007": "release checklist item; no baseline change has happened; the mechanism is the index's notes_url and the UI's update text, to be stated in the release that announces an end of support",
    "ENK-FOOT-004": "openwrt/build-image.sh reports the image sizes: ELECOM 7,668,278 bytes of 24 MiB, Buffalo 5,701,922 of 7,936 KiB; against the stock image: M-3 part 1 measured the constrained class under stock (5,696 vs 5,888 KB); open: the standard class against stock when the device checks run",
    "ENK-FOOT-006": "open: M-3 part 2 on the device; M-2 measured the daemon's resident memory on the VM within the budget (docs/src/measurements/m-2-daemon-footprint.md)",
    "ENK-FOOT-007": "open: M-3 part 2 on the device; M-2 on the VM",
    "ENK-FOOT-008": "open: M-3 part 2 on the device; the argon2 parameters per profile (openwrt/profiles, 8 MiB) keep the transient under 12 MB by construction",
    "ENK-FOOT-010": "open: M-3 part 2 on the slowest Tier 1 device; emulation gave 182 ms (not representative)",
    "ENK-RF-005": "open: the owner's inquiry (chapter 5 §5.4); no image is published; docs/src/releases/0.3.0.md and the install guides say so",
    "ENK-BUILD-007": "openwrt/lock/inputs.json (Image Builder and SDK archives with SHA-256, the Rust nightly, Node, the squashfs settings, the budgets); openwrt/lock/packages/PROFILE.json (every OpenWrt package with version, path and SHA-256, written by openwrt/lock-packages.sh); Cargo.lock, ui/package-lock.json, openwrt/patches/ committed; docs/src/design/build.md §7",
    "ENK-BUILD-008": "openwrt/build-image.sh: the locked packages fetched by path (ENKRNETT_MIRROR or the download tree), checked against the lock, placed in the Image Builder's packages/ with an empty repositories file; .github/workflows/ci.yml job images builds the Tier 1 profiles that way, vm.yml the x86-64 successors; --live exists for the test VM only and openwrt/release-assemble.sh refuses its output",
    "ENK-BUILD-009": "openwrt/build-mips.sh --twice (the daemon, identical 2026-11-27) and the ui job's second vite build with diff -r (identical), both on every CI run (.github/workflows/ci.yml)",
    "ENK-BUILD-010": "openwrt/build-image.sh --twice compares every image of a profile and records built_twice_identical in the build report; CI job images does it for both Tier 1 profiles; openwrt/release-assemble.sh writes build-report.md for the release notes; 2026-11-27: every image identical",
    "ENK-BUILD-011": "openwrt/build-mips.sh, .github/workflows/ci.yml",
    "ENK-BUILD-012": "openwrt/release-assemble.sh: images, SHA256SUMS/SHA512SUMS, lock/ (inputs, package locks, Cargo and UI locks, patches), source-offer.md (OpenWrt's tag and revision, the feed commits of the lock, where the packages came from), build-report.md, index.json; the signatures are the signing host's (docs/src/contributors/releasing.md §4, §5)",
}


def requirements():
    rows = []
    for f in sorted(REQ_DIR.glob("*.md")):
        if f.name == "index-table.md":
            continue
        for line in f.read_text().splitlines():
            m = re.match(r"^\| (ENK-[A-Z]+-\d{3}) \| .* \| ([^|]*) \|$", line)
            if m:
                rows.append((m.group(1), m.group(2).strip()))
    return rows


def mentions():
    found = {}
    for pattern in TEST_GLOBS:
        for f in ROOT.glob(pattern):
            if "node_modules" in f.parts or f.name.endswith(".md"):
                continue
            text = f.read_text(errors="replace")
            for rid in set(re.findall(r"ENK-[A-Z]+-\d{3}", text)):
                found.setdefault(rid, set()).add(str(f.relative_to(ROOT)))
            # "ENK-OWN-001 to 008" and "ENK-OWN-009, 010, 012" forms
            for area, first, rest in re.findall(r"(ENK-[A-Z]+)-(\d{3})((?:,? (?:to |and )?\d{3})+)", text):
                for n in re.findall(r"\d{3}", rest):
                    found.setdefault(f"{area}-{n}", set()).add(str(f.relative_to(ROOT)))
                for n in range(int(first), int(first) + 1):
                    found.setdefault(f"{area}-{n:03d}", set()).add(str(f.relative_to(ROOT)))
            for area, a, b in re.findall(r"(ENK-[A-Z]+)-(\d{3}) to (\d{3})", text):
                for n in range(int(a), int(b) + 1):
                    found.setdefault(f"{area}-{n:03d}", set()).add(str(f.relative_to(ROOT)))
    return found


def main():
    release = None
    if len(sys.argv) > 2 and sys.argv[1] == "--release":
        release = sys.argv[2]
    reqs = requirements()
    found = mentions()
    out = ["# Requirement to test traceability", "", "Generated by `tests/traceability.py` from the requirement tables and the test files that name", "each identifier (chapter 14 rule 1). Inspection notes come from the script's table.", "", "| ID | Release | Tests that name it | Inspection or measurement |", "|---|---|---|---|"]
    uncovered = []
    for rid, rel in reqs:
        tests = sorted(found.get(rid, []))
        note = INSPECTION.get(rid, "")
        if not tests and not note and release and rel == release:
            uncovered.append(rid)
        out.append(f"| {rid} | {rel} | {'<br>'.join(f'`{t}`' for t in tests)} | {note} |")
    covered = sum(1 for rid, _ in reqs if found.get(rid) or rid in INSPECTION)
    out.insert(5, f"{covered} of {len(reqs)} requirements have a test or an inspection note.")
    out.insert(6, "")
    dest = ROOT / "docs/src/verification/traceability.md"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text("\n".join(out) + "\n")
    print(f"{covered}/{len(reqs)} covered; table written to {dest.relative_to(ROOT)}")
    if uncovered:
        print(f"uncovered for {release}: {', '.join(uncovered)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
