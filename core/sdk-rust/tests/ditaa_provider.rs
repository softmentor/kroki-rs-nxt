use kroki_core::{DiagramProvider, DiagramRequest, DiagramOptions, OutputFormat, DitaaProvider, DiagramError};

#[tokio::test]
async fn ditaa_provider_rejects_empty_source() {
    let provider = DitaaProvider::new();
    let request = DiagramRequest {
        source: "   \n  \t ".to_string(),
        diagram_type: "ditaa".to_string(),
        options: DiagramOptions::default(),
        output_format: OutputFormat::Svg,
    };

    let result = provider.generate(&request).await;
    assert!(matches!(result, Err(DiagramError::ValidationFailed(_))));
}

#[tokio::test]
async fn ditaa_provider_maps_missing_binary_to_tool_not_found() {
    let provider = DitaaProvider::with_binary("this-binary-does-not-exist");
    let request = DiagramRequest {
        source: "A -> B".to_string(),
        diagram_type: "ditaa".to_string(),
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
