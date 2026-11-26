//! ENK-RLS-001, ENK-RLS-002, ENK-RLS-006, ENK-UPD-009: the index format and what it offers.

use super::*;

fn entry(profile: &str, version: &str, from: &str, to: &str) -> ReleaseEntry {
    ReleaseEntry {
        profile: profile.into(),
        version: version.into(),
        baseline: to.into(),
        from_baseline: from.into(),
        file: format!("enkrnett-{version}-{to}-{profile}-sysupgrade.bin"),
        size: 6_000_000,
        sha512: "ab".repeat(64),
        min_free_kib: 24_576,
        notes_url: format!("https://example.org/releases/{version}"),
        install_seconds: 180,
    }
}

fn index(releases: Vec<ReleaseEntry>) -> ReleaseIndex {
    ReleaseIndex {
        format: 1,
        sequence: 7,
        releases,
    }
}

fn v(s: &str) -> Version {
    Version::parse(s).unwrap()
}

#[test]
fn versions_parse_strictly_and_compare_numerically() {
    assert_eq!(
        v("0.2.0"),
        Version {
            major: 0,
            minor: 2,
            patch: 0
        }
    );
    assert!(v("0.10.0") > v("0.9.9"));
    assert!(v("1.0.0") > v("0.99.99"));
    assert_eq!(v("1.2.3").to_string(), "1.2.3");
    for bad in [
        "1.2",
        "1.2.3.4",
        "v1.2.3",
        "1.02.3",
        "1.2.3-rc1",
        "",
        "1..3",
        "1.2.1234567",
    ] {
        assert_eq!(Version::parse(bad), None, "{bad}");
    }
}

#[test]
fn a_well_formed_index_round_trips_and_tolerates_unknown_fields() {
    let idx = index(vec![entry("x86_64_generic", "0.3.0", "25.12", "25.12")]);
    let json = serde_json::to_vec(&idx).unwrap();
    assert_eq!(ReleaseIndex::parse(&json), Ok(idx.clone()));
    let mut value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    value["later_field"] = serde_json::json!({"a": 1});
    value["releases"][0]["later_too"] = serde_json::json!(true);
    assert_eq!(
        ReleaseIndex::parse(&serde_json::to_vec(&value).unwrap()),
        Ok(idx)
    );
}

#[test]
fn malformed_indexes_are_refused_with_the_field() {
    assert_eq!(ReleaseIndex::parse(b"{"), Err(IndexError::Json));
    assert_eq!(ReleaseIndex::parse(b"[]"), Err(IndexError::Json));
    assert_eq!(
        ReleaseIndex::parse(br#"{"format":2,"sequence":1,"releases":[]}"#),
        Err(IndexError::Format(2))
    );
    assert_eq!(
        ReleaseIndex::parse(br#"{"format":1,"sequence":-1,"releases":[]}"#),
        Err(IndexError::Json)
    );
    let big = vec![0u8; INDEX_MAX_BYTES + 1];
    assert_eq!(ReleaseIndex::parse(&big), Err(IndexError::Json));
    let many = index(vec![
        entry("p", "0.3.0", "25.12", "25.12");
        INDEX_MAX_RELEASES + 1
    ]);
    assert_eq!(
        ReleaseIndex::parse(&serde_json::to_vec(&many).unwrap()),
        Err(IndexError::TooManyReleases)
    );
    type Mutation = Box<dyn Fn(&mut ReleaseEntry)>;
    let cases: Vec<(&str, Mutation)> = vec![
        ("profile", Box::new(|e| e.profile = "has space".into())),
        ("profile", Box::new(|e| e.profile = String::new())),
        ("version", Box::new(|e| e.version = "0.3".into())),
        ("baseline", Box::new(|e| e.baseline = "25.12;".into())),
        (
            "file",
            Box::new(|e| e.file = "other-0.3.0-25.12-p-sysupgrade.bin".into()),
        ),
        (
            "file",
            Box::new(|e| e.file = "enkrnett-0.3.0-25.12-p-sysupgrade.img".into()),
        ),
        (
            "file",
            Box::new(|e| e.file = "enkrnett-0.3.0-25.12-p-.bin".into()),
        ),
        (
            "file",
            Box::new(|e| e.file = "enkrnett-0.3.0-25.12-p-../x.bin".into()),
        ),
        ("size", Box::new(|e| e.size = 0)),
        ("size", Box::new(|e| e.size = IMAGE_MAX_BYTES + 1)),
        ("sha512", Box::new(|e| e.sha512 = "AB".repeat(64))),
        ("sha512", Box::new(|e| e.sha512 = "ab".repeat(32))),
        ("min_free_kib", Box::new(|e| e.min_free_kib = 1 << 21)),
        (
            "notes_url",
            Box::new(|e| e.notes_url = "http://example.org".into()),
        ),
        (
            "notes_url",
            Box::new(|e| e.notes_url = "https://example.org/a b".into()),
        ),
        ("install_seconds", Box::new(|e| e.install_seconds = 0)),
        ("install_seconds", Box::new(|e| e.install_seconds = 3601)),
    ];
    for (field, mutate) in cases {
        let mut e = entry("p", "0.3.0", "25.12", "25.12");
        mutate(&mut e);
        let idx = index(vec![e]);
        assert_eq!(
            ReleaseIndex::parse(&serde_json::to_vec(&idx).unwrap()),
            Err(IndexError::Field { entry: 0, field }),
            "{field}"
        );
    }
}

#[test]
fn the_offer_is_the_highest_newer_release_for_this_profile_and_baseline() {
    let idx = index(vec![
        entry("p", "0.3.0", "25.12", "25.12"),
        entry("p", "0.3.1", "25.12", "25.12"),
        entry("p", "0.2.0", "25.12", "25.12"),
        entry("q", "0.9.0", "25.12", "25.12"),
        entry("p", "0.4.0", "26.06", "26.06"),
    ]);
    assert_eq!(
        idx.offer("p", v("0.2.0"), "25.12")
            .map(|r| r.version.as_str()),
        Some("0.3.1")
    );
    // Up to date, or ahead of the index (ENK-UPD-009): nothing is offered.
    assert_eq!(idx.offer("p", v("0.3.1"), "25.12"), None);
    assert_eq!(idx.offer("p", v("0.5.0"), "25.12"), None);
    // Another profile's releases and another baseline's are not for this device.
    assert_eq!(idx.offer("r", v("0.1.0"), "25.12"), None);
    assert_eq!(idx.offer("p", v("0.1.0"), "24.10"), None);
}

#[test]
fn an_upgrade_is_offered_from_its_previous_baseline_and_says_so() {
    let idx = index(vec![
        entry("p", "0.5.0", "25.12", "26.06"),
        entry("p", "0.4.0", "25.12", "25.12"),
    ]);
    let r = idx.offer("p", v("0.2.0"), "25.12").unwrap();
    assert_eq!(r.version.as_str(), "0.5.0");
    assert!(r.changes_baseline());
    let a = r.available();
    assert_eq!(
        (a.changes_baseline, a.baseline.as_str(), a.size_kib),
        (true, "26.06", 5860)
    );
    // The same version offered both as an upgrade and within the baseline: the latter wins.
    let idx = index(vec![
        entry("p", "0.5.0", "25.12", "26.06"),
        entry("p", "0.5.0", "25.12", "25.12"),
    ]);
    assert!(
        !idx.offer("p", v("0.2.0"), "25.12")
            .unwrap()
            .changes_baseline()
    );
}

#[test]
fn the_install_checks_name_the_first_failing_condition_and_the_image_sits_beside_the_index() {
    use crate::api::UpdateFailure;
    let e = entry("x86_64_generic", "0.3.0", "25.12", "25.12");
    assert_eq!(
        e.image_url("https://updates.example/enkrnett/index.json"),
        format!("https://updates.example/enkrnett/{}", e.file)
    );
    // ENK-UPD-003: the image plus 16 MB, or the index's own minimum when that is larger (the
    // fixture asks for 24 MiB).
    let kib = e.size.div_ceil(1024);
    assert_eq!(e.needed_kib(), kib + 24_576);
    assert_eq!(
        e.check_device("x86_64_generic", "25.12", kib + 24_576),
        Ok(())
    );
    assert_eq!(
        e.check_device("x86_64_generic", "25.12", kib + 24_576 - 1),
        Err(UpdateFailure::NotEnoughMemory)
    );
    let mut small = e.clone();
    small.min_free_kib = 1024;
    assert_eq!(small.needed_kib(), kib + 16 * 1024);
    assert_eq!(
        e.check_device("other_device", "25.12", 1 << 20),
        Err(UpdateFailure::WrongDevice)
    );
    assert_eq!(
        e.check_device("x86_64_generic", "24.10", 1 << 20),
        Err(UpdateFailure::WrongBaseline)
    );
    let mut big = e.clone();
    big.min_free_kib = 40_000;
    assert_eq!(big.needed_kib(), kib + 40_000);
    // The bytes: the size and the hash the index states, both.
    assert_eq!(e.check_bytes(e.size, &e.sha512), Ok(()));
    assert_eq!(
        e.check_bytes(e.size - 1, &e.sha512),
        Err(UpdateFailure::BadSignature)
    );
    assert_eq!(
        e.check_bytes(e.size, &"00".repeat(64)),
        Err(UpdateFailure::BadSignature)
    );
}
