use kroki_adapter_transport as _;

#[test]
fn transport_adapter_public_surface_is_linkable() {
    let crate_name = env!("CARGO_PKG_NAME");
    assert_eq!(crate_name, "kroki-adapter-transport");
}
