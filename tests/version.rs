use rrd::ops::version;

#[test]
fn version_seems_reasonable() {
    let vers = version::librrd_version();
    assert!(!vers.is_empty());
    assert!(vers.starts_with("1."), "{}", vers);
}
