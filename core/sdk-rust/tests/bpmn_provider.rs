use kroki_core::{
    BpmnProvider, DiagramOptions, DiagramProvider, DiagramRequest, DiagramResult, OutputFormat,
};

fn request(source: &str) -> DiagramRequest {
    DiagramRequest {
        source: source.to_string(),
        diagram_type: "bpmn".to_string(),
        output_format: OutputFormat::Svg,
        options: DiagramOptions::default(),
    }
}

#[tokio::test]
async fn bpmn_provider_rejects_empty_source() {
    let provider = BpmnProvider::new();
    let result: DiagramResult<_> = provider.generate(&request("   ")).await;
    assert!(result.is_err());
}

#[cfg(not(feature = "native-browser"))]
#[tokio::test]
async fn bpmn_provider_reports_feature_gate_without_native_browser() {
    let provider = BpmnProvider::new();
    let result = provider
        .generate(&request(
            "<?xml version=\"1.0\"?><definitions></definitions>",
        ))
        .await
        .expect_err("without native-browser feature, bpmn should report ToolNotFound");
    assert_eq!(
        result.to_string(),
        "Tool not found: native-browser feature disabled for bpmn provider"
    );
}

