use kroki_core::{DiagramProvider, DiagramRequest, DiagramOptions, OutputFormat, WavedromProvider, DiagramError};

#[tokio::test]
async fn wavedrom_provider_rejects_empty_source() {
    let provider = WavedromProvider::new();
    let request = DiagramRequest {
        source: "   \n  \t ".to_string(),
        diagram_type: "wavedrom".to_string(),
        options: DiagramOptions::default(),
        output_format: OutputFormat::Svg,
    };

    let result = provider.generate(&request).await;
    assert!(matches!(result, Err(DiagramError::ValidationFailed(_))));
}

#[tokio::test]
async fn wavedrom_provider_maps_missing_binary_to_tool_not_found() {
    let provider = WavedromProvider::with_binary("this-binary-does-not-exist");
    let request = DiagramRequest {
        source: "{\"signal\":[{\"name\":\"clk\",\"wave\":\"p\"}]}".to_string(),
        diagram_type: "wavedrom".to_string(),
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
