#!/usr/bin/env python3
"""Requirement-to-test traceability (chapter 14 rule 1).

Every test names the requirement identifiers it verifies in its file header or function name.
This script lists, for every requirement of the specification, the test files that name it, and
writes the table to docs/src/verification/traceability.md. It exits non-zero when a requirement
allocated to the release given with --release has no test and no inspection note.

    tests/traceability.py            write the table
    tests/traceability.py --release 0.1.0   also fail on uncovered 0.1.0 requirements
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
    "ENK-SEC-014": "crates/enkrnettd: no std::process::Command; crates/enkrnett-ubus speaks ubus directly",
    "ENK-SEC-015": "crates/enkrnettd/src/http/response.rs: no-store on every API response",
    "ENK-SEC-016": "crates/enkrnettd/src/log.rs and app.rs: events carry no key, password, or token",
    "ENK-API-027": "crates/enkrnettd/src/app.rs status reads the platform per request; the 2 s cache arrives with M2's effective-state cache",
    "ENK-OWN-022": "tests/browser/tests/setup.spec.ts (the connection line on S-1); ui/src/components/ConnectionLine.svelte",
    "ENK-UI-018": "ui/src/lib/countdown.svelte.ts (performance.now); tests/browser setup.spec.ts shows the countdown",
    "ENK-UI-022": "ui components render API values as text; the only {@html} is the QR SVG; hostile-value test to add to the harness",
    "ENK-FOOT-001": "openwrt/build-mips.sh measured 256 KiB (xz estimate) of 300",
    "ENK-FOOT-011": "browser harness: timing check to add; the bundle is 44 KiB compressed",
    "ENK-RF-003": "openwrt/profiles: every product profile fixes the country, so no radio transmits before it is set",
    "ENK-REPO-002": "openwrt/patches/0001-…: header with upstream reference and removal condition",
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
    "ENK-BUILD-009": "CI (planned double build)",
    "ENK-BUILD-011": "openwrt/build-mips.sh, .github/workflows/ci.yml",
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
