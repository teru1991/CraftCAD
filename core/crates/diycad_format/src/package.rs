use crate::{validate_entry_path, LimitViolation, Limits, ReasonCode};
use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct ZipIndexEntry {
    pub path: String,
    pub uncompressed_size: u64,
}

#[derive(Debug)]
pub struct PackageReader<R: Read + Seek> {
    zip: ZipArchive<R>,
    pub entries: Vec<ZipIndexEntry>,
    pub total_uncompressed: u64,
}

impl<R: Read + Seek> PackageReader<R> {
    pub fn open(mut zip: ZipArchive<R>, limits: &Limits) -> Result<Self> {
        let mut entries: Vec<ZipIndexEntry> = Vec::new();
        let mut total: u64 = 0;

        let n = zip.len();
        if n > limits.max_entries {
            return Err(anyhow!(
                "{}: too many entries: {} > {}",
                ReasonCode::SecZipTooManyEntries.as_str(),
                n,
                limits.max_entries
            ));
        }

        for i in 0..n {
            let f = zip.by_index(i).with_context(|| "zip.by_index failed")?;
            let norm = match validate_entry_path(limits, f.name()) {
                Ok(p) => p,
                Err(LimitViolation { code, message }) => {
                    return Err(anyhow!("{}: {}", code.as_str(), message));
                }
            };
            if f.is_dir() {
                continue;
            }
            let size = f.size();
            if size > limits.max_entry_uncompressed {
                return Err(anyhow!(
                    "{}: entry too large: {} ({} bytes) > {}",
                    ReasonCode::SecZipEntryTooLarge.as_str(),
                    norm,
                    size,
                    limits.max_entry_uncompressed
                ));
            }
            total = total.saturating_add(size);
            if total > limits.max_total_uncompressed {
                return Err(anyhow!(
                    "{}: total uncompressed too large: {} > {}",
                    ReasonCode::SecZipTotalUncompressedTooLarge.as_str(),
                    total,
                    limits.max_total_uncompressed
                ));
            }
            entries.push(ZipIndexEntry {
                path: norm,
                uncompressed_size: size,
            });
        }

        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self {
            zip,
            entries,
            total_uncompressed: total,
        })
    }

    pub fn read_entry_bytes(&mut self, path: &str, limit: u64) -> Result<Option<Vec<u8>>> {
        if !self.entries.iter().any(|e| e.path == path) {
            return Ok(None);
        }
        let mut f = self
            .zip
            .by_name(path)
            .with_context(|| format!("zip.by_name failed: {}", path))?;
        let sz = f.size();
        if sz > limit {
            return Err(anyhow!(
                "{}: read limit exceeded for {} ({} > {})",
                ReasonCode::SecZipEntryTooLarge.as_str(),
                path,
                sz,
                limit
            ));
        }
        let mut buf = Vec::with_capacity(sz as usize);
        f.read_to_end(&mut buf)
            .with_context(|| format!("read_to_end failed: {}", path))?;
        Ok(Some(buf))
    }
}

pub fn open_package_file(path: &Path, limits: &Limits) -> Result<PackageReader<File>> {
    let f = File::open(path).with_context(|| format!("open failed: {}", path.display()))?;
    let zip =
        ZipArchive::new(f).map_err(|e| anyhow!("{}: {}", ReasonCode::SecZipBadZip.as_str(), e))?;
    PackageReader::open(zip, limits)
}
