//! Artefact cache (PRD §6.1): URL sources are downloaded to a cache and
//! verified against a required sha256 before use; local paths pass through.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::config::model::ArtefactSource;

/// `~/.local/share/vmlab/cache/artefacts`
pub fn cache_dir() -> PathBuf {
    crate::paths::data_dir().join("cache").join("artefacts")
}

/// Resolve an artefact source to a local file path, downloading + verifying
/// URL sources. Reports progress via `log`.
pub async fn resolve(source: &ArtefactSource, log: impl Fn(String)) -> Result<PathBuf> {
    match source {
        ArtefactSource::Path { path, .. } => {
            if !path.is_file() {
                bail!("source file {} does not exist", path.display());
            }
            Ok(path.clone())
        }
        ArtefactSource::Url { url, sha256, .. } => {
            let dir = cache_dir();
            std::fs::create_dir_all(&dir)?;
            // Cache key = the expected digest, so a re-download is skipped
            // when the verified artefact is already present.
            let cached = dir.join(format!("{sha256}.artefact"));
            if cached.is_file() && verify_file(&cached, sha256).await.is_ok() {
                log(format!("using cached artefact {sha256}"));
                return Ok(cached);
            }
            log(format!("downloading {url}"));
            download(url, &cached).await?;
            verify_file(&cached, sha256)
                .await
                .with_context(|| format!("hash mismatch for {url}"))?;
            log(format!("verified sha256 {sha256}"));
            Ok(cached)
        }
    }
}

async fn download(url: &str, dest: &Path) -> Result<()> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;

    let tmp = dest.with_extension("part");
    let resp = reqwest::get(url)
        .await
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        bail!("GET {url} returned {}", resp.status());
    }
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

async fn verify_file(path: &Path, expected_hex: &str) -> Result<()> {
    let path = path.to_path_buf();
    let expected = expected_hex.to_ascii_lowercase();
    let actual = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut f = std::fs::File::open(&path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut f, &mut hasher)?;
        Ok(hex::encode(hasher.finalize()))
    })
    .await??;
    if actual != expected {
        bail!("sha256 mismatch: expected {expected}, got {actual}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_path_passthrough() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.iso");
        std::fs::write(&f, b"hello").unwrap();
        let src = ArtefactSource::Path {
            path: f.clone(),
            span: (0, 0),
        };
        let resolved = resolve(&src, |_| {}).await.unwrap();
        assert_eq!(resolved, f);
    }

    #[tokio::test]
    async fn missing_local_path_errors() {
        let src = ArtefactSource::Path {
            path: PathBuf::from("/no/such.iso"),
            span: (0, 0),
        };
        assert!(resolve(&src, |_| {}).await.is_err());
    }

    #[tokio::test]
    async fn verify_detects_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x");
        std::fs::write(&f, b"hello").unwrap();
        // sha256("hello")
        let good = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        verify_file(&f, good).await.unwrap();
        assert!(verify_file(&f, &"0".repeat(64)).await.is_err());
    }
}
