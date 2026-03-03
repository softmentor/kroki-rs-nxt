use kroki_core::{
    D2Provider, DiagramOptions, DiagramProvider, DiagramRequest, DiagramResult, OutputFormat,
};

fn request(source: &str) -> DiagramRequest {
    DiagramRequest {
        source: source.to_string(),
        diagram_type: "d2".to_string(),
        output_format: OutputFormat::Svg,
        options: DiagramOptions::default(),
    }
}

#[tokio::test]
async fn d2_provider_rejects_empty_source() {
    let provider = D2Provider::new();
    let result: DiagramResult<_> = provider.generate(&request("   ")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn d2_provider_maps_missing_binary_to_tool_not_found() {
    let provider = D2Provider::with_binary("kroki-d2-missing-for-test");
    let result = provider
        .generate(&request("a -> b"))
        .await
        .expect_err("missing tool should return DiagramError");

    assert_eq!(
        result.to_string(),
        "Tool not found: kroki-d2-missing-for-test"
    );
}

#[tokio::test]
async fn d2_provider_renders_svg_when_binary_available() {
    if which::which("d2").is_err() {
        return;
    }

    let provider = D2Provider::new();
    let output = provider
        .generate(&request("a -> b"))
        .await
        .expect("d2 render should succeed when binary is installed");

    assert_eq!(output.content_type, "image/svg+xml");
    assert!(
        String::from_utf8_lossy(&output.data).contains("<svg"),
        "d2 output should contain SVG tag"
    );
}
