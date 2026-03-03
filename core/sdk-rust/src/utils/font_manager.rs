use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use hex::encode as hex_encode;
use reqwest::Client;
use sha2::{Digest, Sha256};
use tokio::fs;

/// Downloads and caches custom fonts for browser rendering harness injection.
pub struct FontManager {
    cache_dir: PathBuf,
    client: Client,
}

impl FontManager {
    pub fn new() -> Result<Self> {
        let base = std::env::var("KROKI_FONT_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".kroki-cache/fonts"));
        std::fs::create_dir_all(&base)?;

        Ok(Self {
            cache_dir: base,
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .context("failed to construct font HTTP client")?,
        })
    }

    pub async fn prepare_font_css(&self, urls: &[String]) -> Result<Option<String>> {
        if urls.is_empty() {
            return Ok(None);
        }

        let mut css = String::new();
        for url in urls {
            let local = self.fetch_or_cache(url).await?;
            let family = Self::font_family_from_url(url);
            let escaped = local.to_string_lossy().replace('\\', "\\\\");
            css.push_str(&format!(
                "@font-face{{font-family:'{}';src:url('file://{}');}}\n",
                family, escaped
            ));
        }
        Ok(Some(css))
    }

    async fn fetch_or_cache(&self, source: &str) -> Result<PathBuf> {
        let file_name = Self::safe_file_name(source);
        let target = self.cache_dir.join(file_name);
        if target.exists() {
            return Ok(target);
        }

        if source.starts_with("http://") || source.starts_with("https://") {
            let resp = self.client.get(source).send().await?.error_for_status()?;
            let bytes = resp.bytes().await?;
            Self::validate_font_size(bytes.len())?;
            fs::write(&target, &bytes).await?;
        } else {
            let path = Path::new(source);
            let bytes = fs::read(path).await?;
            Self::validate_font_size(bytes.len())?;
            fs::write(&target, &bytes).await?;
        }
        Ok(target)
    }

    fn safe_file_name(source: &str) -> String {
        let hash = Sha256::digest(source.as_bytes());
        let mut name = hex_encode(hash);

        let ext = source
            .rsplit('/')
            .next()
            .and_then(|segment| segment.split('?').next())
            .and_then(|segment| segment.split('#').next())
            .and_then(|segment| Path::new(segment).extension().and_then(|e| e.to_str()))
            .unwrap_or("ttf");
        name.push('.');
        name.push_str(&ext.to_lowercase());
        name
    }

    fn font_family_from_url(source: &str) -> String {
        source
            .rsplit('/')
            .next()
            .and_then(|segment| segment.split('?').next())
            .and_then(|segment| segment.split('#').next())
            .and_then(|segment| Path::new(segment).file_stem())
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("kroki-custom-font")
            .replace(['-', '_'], " ")
    }

    fn validate_font_size(actual: usize) -> Result<()> {
        const MAX_FONT_BYTES: usize = 5 * 1024 * 1024;
        if actual > MAX_FONT_BYTES {
            anyhow::bail!(
                "font payload too large ({} bytes). maximum allowed is {} bytes",
                actual,
                MAX_FONT_BYTES
            );
        }
        Ok(())
    }
}
