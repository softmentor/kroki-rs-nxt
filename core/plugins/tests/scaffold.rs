use kroki_plugins as _;

#[test]
fn plugin_crate_public_surface_is_linkable() {
    let crate_name = env!("CARGO_PKG_NAME");
    assert_eq!(crate_name, "kroki-plugins");
}
