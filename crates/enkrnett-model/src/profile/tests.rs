//! ENK-TIER-001, 002, 004, 005: the shipped profiles parse and meet the rules.

use super::*;

const WRC_1167GST2: &str =
    include_str!("../../../../openwrt/profiles/elecom_wrc-1167gst2/profile.json");
const WCR_1166DS: &str =
    include_str!("../../../../openwrt/profiles/buffalo_wcr-1166ds/profile.json");

#[test]
fn the_shipped_profiles_parse_and_pass() {
    for (name, json) in [("wrc-1167gst2", WRC_1167GST2), ("wcr-1166ds", WCR_1166DS)] {
        let p = DeviceProfile::parse(json).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(p.check(), vec![], "{name}");
        assert_eq!(p.tier, 2, "{name}: unverified profiles are Tier 2");
        assert!(!p.verified.on_hardware);
        assert_eq!(p.country.fixed.as_deref(), Some("JP"));
        assert_eq!(p.package_arch, "mipsel_24kc");
    }
}

#[test]
fn classes_match_the_hardware() {
    let standard = DeviceProfile::parse(WRC_1167GST2).unwrap();
    assert_eq!(standard.device_class, DeviceClass::Standard);
    assert!(standard.platform_hardening_complete);
    assert_eq!(standard.radios[0].interface_limit, 16);
    let constrained = DeviceProfile::parse(WCR_1166DS).unwrap();
    assert_eq!(constrained.device_class, DeviceClass::Constrained);
    assert!(!constrained.platform_hardening_complete);
    assert_eq!(constrained.flash_image_limit_kib, 7936);
    assert_eq!(constrained.ram_mib, 64);
}

#[test]
fn tier_one_needs_hardware_verification() {
    let mut p = DeviceProfile::parse(WRC_1167GST2).unwrap();
    p.tier = 1;
    let errors = p.check();
    assert!(errors.iter().any(|e| e.0.contains("ENK-TIER-004")));
    let mut c = DeviceProfile::parse(WCR_1166DS).unwrap();
    c.tier = 1;
    c.verified.on_hardware = true;
    assert!(c.check().iter().any(|e| e.0.contains("ENK-TIER-005")));
}

#[test]
fn unknown_fields_and_wrong_formats_are_refused() {
    let with_extra =
        WRC_1167GST2.replacen("\"format\": 1,", "\"format\": 1, \"colour\": \"red\",", 1);
    assert!(DeviceProfile::parse(&with_extra).is_err());
    let wrong_format = WRC_1167GST2.replacen("\"format\": 1,", "\"format\": 2,", 1);
    assert!(DeviceProfile::parse(&wrong_format).is_err());
}

#[test]
fn rules_catch_impossible_values() {
    let mut p = DeviceProfile::parse(WRC_1167GST2).unwrap();
    p.radios.clear();
    p.argon2.iterations = 0;
    p.country.fixed = Some("Japan".into());
    p.id = "Bad Id".into();
    let errors = p.check();
    assert!(errors.len() >= 4, "{errors:?}");
}
