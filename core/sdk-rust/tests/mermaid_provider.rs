use kroki_core::{
    DiagramOptions, DiagramProvider, DiagramRequest, DiagramResult, MermaidProvider, OutputFormat,
};

fn request(source: &str) -> DiagramRequest {
    DiagramRequest {
        source: source.to_string(),
        diagram_type: "mermaid".to_string(),
        output_format: OutputFormat::Svg,
        options: DiagramOptions::default(),
    }
}

#[tokio::test]
async fn mermaid_provider_rejects_empty_source() {
    let provider = MermaidProvider::new();
    let result: DiagramResult<_> = provider.generate(&request("   ")).await;
    assert!(result.is_err());
}

#[cfg(not(feature = "native-browser"))]
#[tokio::test]
async fn mermaid_provider_reports_feature_gate_when_browser_runtime_disabled() {
    let provider = MermaidProvider::new();
    let result = provider
        .generate(&request("graph TD; A-->B;"))
        .await
        .expect_err("without native-browser feature, mermaid should report ToolNotFound");
    assert_eq!(
        result.to_string(),
        "Tool not found: native-browser feature disabled for mermaid provider"
    );
}

#[cfg(feature = "native-browser")]
#[tokio::test]
async fn mermaid_provider_maps_missing_binary_when_enabled() {
    let provider = MermaidProvider::with_binary("kroki-mmdc-missing-for-test");
    let result = provider
        .generate(&request("graph TD; A-->B;"))
        .await
        .expect_err("missing mmdc should map to ToolNotFound");
    assert_eq!(
        result.to_string(),
        "Tool not found: kroki-mmdc-missing-for-test"
    );
}

#[cfg(feature = "native-browser")]
#[tokio::test]
async fn mermaid_provider_renders_svg_when_mmdc_available() {
    if which::which("mmdc").is_err() {
        return;
    }

    let provider = MermaidProvider::new();
    let output = provider
        .generate(&request("graph TD; A-->B;"))
        .await
        .expect("mermaid should render when mmdc is installed");
    assert_eq!(output.content_type, "image/svg+xml");
    assert!(String::from_utf8_lossy(&output.data).contains("<svg"));
}
