//! kroki-adapter-storage: Storage implementations for the kroki platform.
//!
//! Provides:
//! - Filesystem-based diagram cache (SHA256-keyed)
//! - Cache lookup, storage, and invalidation
//! - Future: database-backed storage, cloud storage adapters
//!
//! Bootstrap baseline storage adapter crate; feature-complete implementation is planned for Phase 3 (Batch 4+5).

#[cfg(test)]
mod tests {
    #[test]
    fn unit_storage_adapter_crate_name_is_stable() {
        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "kroki-adapter-storage");
    }
}
