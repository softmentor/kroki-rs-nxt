use dashmap::DashMap;
use kroki_core::config::CircuitBreakerConfig;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone)]
pub struct CircuitBreakerManager {
    circuits: Arc<DashMap<String, ProviderCircuit>>,
    config: CircuitBreakerConfig,
}

struct ProviderCircuit {
    state: CircuitState,
    consecutive_failures: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    reset_timeout_secs: u64,
}

impl ProviderCircuit {
    fn new(config: &CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            last_failure_time: None,
            failure_threshold: config.failure_threshold.max(1),
            reset_timeout_secs: config.reset_timeout_secs,
        }
    }

    fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed().as_secs() >= self.reset_timeout_secs {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => false,
        }
    }

    fn record_success(&mut self) {
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.last_failure_time = None;
    }

    fn record_failure(&mut self) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.last_failure_time = Some(Instant::now());

        if self.state == CircuitState::HalfOpen
            || self.consecutive_failures >= self.failure_threshold
        {
            self.state = CircuitState::Open;
        }
    }
}

impl CircuitBreakerManager {
    pub fn new(config: &CircuitBreakerConfig) -> Self {
        Self {
            circuits: Arc::new(DashMap::new()),
            config: config.clone(),
        }
    }

    pub fn should_allow(&self, provider: &str) -> bool {
        let mut entry = self
            .circuits
            .entry(provider.to_string())
            .or_insert_with(|| ProviderCircuit::new(&self.config));
        entry.should_allow()
    }

    pub fn record_success(&self, provider: &str) {
        if let Some(mut entry) = self.circuits.get_mut(provider) {
            entry.record_success();
        }
    }

    pub fn record_failure(&self, provider: &str) {
        let mut entry = self
            .circuits
            .entry(provider.to_string())
            .or_insert_with(|| ProviderCircuit::new(&self.config));
        entry.record_failure();
    }

    pub fn state(&self, provider: &str) -> CircuitState {
        self.circuits
            .get(provider)
            .map(|entry| entry.state)
            .unwrap_or(CircuitState::Closed)
    }
}
