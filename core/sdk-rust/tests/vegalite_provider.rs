use kroki_core::{DiagramProvider, DiagramRequest, DiagramOptions, OutputFormat, VegaLiteProvider, DiagramError};

#[tokio::test]
async fn vegalite_provider_rejects_empty_source() {
    let provider = VegaLiteProvider::new();
    let request = DiagramRequest {
        source: "   \n  \t ".to_string(),
        diagram_type: "vegalite".to_string(),
        options: DiagramOptions::default(),
        output_format: OutputFormat::Svg,
    };

    let result = provider.generate(&request).await;
    assert!(matches!(result, Err(DiagramError::ValidationFailed(_))));
}

#[tokio::test]
async fn vegalite_provider_maps_missing_binary_to_tool_not_found() {
    let provider = VegaLiteProvider::with_binaries("this-binary-does-not-exist", "vg2svg");
    let request = DiagramRequest {
        source: "{\"data\": []}".to_string(),
        diagram_type: "vegalite".to_string(),
        options: DiagramOptions {
            timeout_ms: Some(500),
            ..Default::default()
        },
        output_format: OutputFormat::Svg,
    };

    let result = provider.generate(&request).await;
    match result {
        Err(DiagramError::ToolNotFound(binary)) => {
            assert_eq!(binary, "this-binary-does-not-exist");
        }
        _ => panic!("Expected ToolNotFound error, got {result:?}"),
    }
}
