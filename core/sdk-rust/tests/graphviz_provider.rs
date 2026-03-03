use kroki_core::{
    DiagramOptions, DiagramProvider, DiagramRequest, DiagramResult, GraphvizProvider, OutputFormat,
};

fn request(source: &str) -> DiagramRequest {
    DiagramRequest {
        source: source.to_string(),
        diagram_type: "graphviz".to_string(),
        output_format: OutputFormat::Svg,
        options: DiagramOptions::default(),
    }
}

#[tokio::test]
async fn graphviz_provider_rejects_empty_source() {
    let provider = GraphvizProvider::new();
    let result: DiagramResult<_> = provider.generate(&request("   ")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn graphviz_provider_maps_missing_binary_to_tool_not_found() {
    let provider = GraphvizProvider::with_binary("kroki-dot-missing-for-test");
    let result = provider
        .generate(&request("digraph G { A -> B; }"))
        .await
        .expect_err("missing tool should return DiagramError");

    assert_eq!(
        result.to_string(),
        "Tool not found: kroki-dot-missing-for-test"
    );
}

#[tokio::test]
async fn graphviz_provider_renders_svg_when_dot_available() {
    if which::which("dot").is_err() {
        return;
    }

    let provider = GraphvizProvider::new();
    let output = provider
        .generate(&request("digraph G { A -> B; }"))
        .await
        .expect("graphviz render should succeed when dot is installed");

    assert_eq!(output.content_type, "image/svg+xml");
    assert!(
        String::from_utf8_lossy(&output.data).contains("<svg"),
        "graphviz output should contain SVG tag"
    );
}

#[tokio::test]
async fn graphviz_provider_accepts_escaped_newline_sequences() {
    if which::which("dot").is_err() {
        return;
    }

    let provider = GraphvizProvider::new();
    let output = provider
        .generate(&request("digraph G {\\nA -> B;\\n}"))
        .await
        .expect("graphviz render should normalize escaped newline sequences");
    assert!(String::from_utf8_lossy(&output.data).contains("<svg"));
}
