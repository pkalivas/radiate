use std::sync::LazyLock;

static ENV_CONFIG: LazyLock<EnvConfig> = LazyLock::new(EnvConfig::from_env);

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub log_level: Option<String>,
    pub log_format: Option<String>,
    pub seed: Option<u64>,
    pub max_threads: Option<usize>,
}

impl EnvConfig {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            log_level: std::env::var("RADIATE_LOG_LEVEL")
                .or_else(|_| std::env::var("RUST_LOG"))
                .ok(),
            log_format: std::env::var("RADIATE_LOG_FORMAT").ok(),
            seed: std::env::var("RADIATE_SEED").ok().and_then(|s| {
                let parsed = s.parse().ok();
                if parsed.is_none() {
                    eprintln!("RADIATE_SEED={s:?} is not a valid u64, ignoring");
                }
                parsed
            }),
            max_threads: std::env::var("RADIATE_MAX_THREADS").ok().and_then(|s| {
                let parsed = s.parse().ok();
                if parsed.is_none() {
                    eprintln!("RADIATE_MAX_THREADS={s:?} is not a valid usize, ignoring");
                }
                parsed
            }),
        }
    }
}

pub fn config() -> &'static EnvConfig {
    &ENV_CONFIG
}

pub fn log_level() -> Option<String> {
    ENV_CONFIG.log_level.clone()
}

pub fn log_format() -> Option<String> {
    ENV_CONFIG.log_format.clone()
}

pub fn seed() -> Option<u64> {
    ENV_CONFIG.seed
}

pub fn max_threads() -> Option<usize> {
    ENV_CONFIG.max_threads
}
