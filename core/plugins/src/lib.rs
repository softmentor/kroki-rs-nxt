//! kroki-plugins: Plugin framework for extending diagram capabilities.
//!
//! Handles:
//! - Plugin discovery from configuration
//! - Subprocess-based plugin protocol (stdin/stdout)
//! - Plugin lifecycle management
//! - Argument templating for plugin commands
//!
//! Bootstrap baseline plugin crate; feature-complete implementation is planned for Phase 3 (Batch 3: Pipeline + Plugins).

#[cfg(test)]
mod tests {
    #[test]
    fn unit_plugin_crate_name_is_stable() {
        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "kroki-plugins");
    }
}
