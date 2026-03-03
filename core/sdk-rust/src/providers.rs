//! Diagram provider implementations.
//!
//! Provider categories:
//! - Command: wraps CLI tools via subprocess (Graphviz, D2, Ditaa, etc.)
//! - Browser: evaluates JS in headless Chrome (Mermaid, BPMN)
//! - Pipeline: multi-step conversion chains (Vega-Lite → Vega → SVG)
//!
//! Bootstrap baseline provider module; concrete providers are planned for Phase 3.
