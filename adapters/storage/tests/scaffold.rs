use kroki_adapter_storage as _;

#[test]
fn storage_adapter_public_surface_is_linkable() {
    let crate_name = env!("CARGO_PKG_NAME");
    assert_eq!(crate_name, "kroki-adapter-storage");
}
