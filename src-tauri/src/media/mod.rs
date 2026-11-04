//! Local sanitized immutable media; source paths never leave this Rust boundary.
use crate::domain::{recipes::valid_id, AppError};
use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

pub const INPUT_BYTES: u64 = 20_000_000;
pub const PIXELS: u64 = 30_000_000;
pub const FULL_BYTES: u64 = 128_000_000;
pub const PREVIEW_BYTES: u64 = 7_000_000;
pub const THUMB_BYTES: u64 = 700_000;
fn decode_error(e: image::ImageError) -> AppError {
    if matches!(e, image::ImageError::Limits(_)) {
        error("MEDIA_LIMIT")
    } else {
        error("MEDIA_INVALID")
    }
}
pub fn error(code: &str) -> AppError {
    let key = match code {
        "MEDIA_LIMIT" => "errors.mediaLimit",
        "MEDIA_UNSUPPORTED" => "errors.mediaUnsupported",
        "MEDIA_BUSY" => "errors.mediaBusy",
        "MEDIA_PATH" => "errors.mediaPath",
        "MEDIA_INVALID" => "errors.mediaInvalid",
        _ => "errors.mediaUnavailable",
    };
    AppError::new(code, key, true)
}
#[derive(Default)]
pub struct ImportGate(AtomicBool);
pub struct ImportPermit<'a>(&'a ImportGate);
impl ImportGate {
    pub fn acquire(&self) -> Result<ImportPermit<'_>, AppError> {
        self.0
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| error("MEDIA_BUSY"))?;
        Ok(ImportPermit(self))
    }
}
impl Drop for ImportPermit<'_> {
    fn drop(&mut self) {
        self.0 .0.store(false, Ordering::SeqCst)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub format_version: u32,
    pub asset_id: String,
    pub width: u32,
    pub height: u32,
    pub variants: Vec<Variant>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedCover {
    pub id: String,
    pub width: u32,
    pub height: u32,
}
impl Manifest {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.asset_id)?;
        if self.format_version != 1
            || self.width == 0
            || self.height == 0
            || u64::from(self.width) * u64::from(self.height) > PIXELS
            || self.variants.len() != 3
        {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        for (v, (name, limit, edge)) in self.variants.iter().zip([
            ("original", FULL_BYTES, 30_000_000),
            ("preview", PREVIEW_BYTES, 1280),
            ("thumb", THUMB_BYTES, 384),
        ]) {
            if v.name != name
                || v.bytes == 0
                || v.bytes > limit
                || v.width == 0
                || v.height == 0
                || v.width > edge
                || v.height > edge
                || v.sha256.len() != 64
                || !v
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(error("MEDIA_UNAVAILABLE"));
            }
        }
        if (self.variants[0].width, self.variants[0].height) != (self.width, self.height) {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct MediaStore {
    root: PathBuf,
}
fn regular(path: &Path) -> Result<(), AppError> {
    let meta = fs::symlink_metadata(path).map_err(|_| error("MEDIA_PATH"))?;
    if meta.file_type().is_symlink() {
        return Err(error("MEDIA_PATH"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(error("MEDIA_PATH"));
        }
    }
    Ok(())
}
fn check_ancestors(path: &Path) -> Result<(), AppError> {
    for p in path.ancestors() {
        if p.as_os_str().is_empty() {
            continue;
        }
        #[cfg(target_os = "macos")]
        {
            if [("/var", "/private/var"), ("/tmp", "/private/tmp")]
                .iter()
                .any(|(a, b)| {
                    p == Path::new(a)
                        && (fs::read_link(p).ok().as_deref() == Some(Path::new(b))
                            || fs::read_link(p).ok().as_deref()
                                == Some(Path::new(b.trim_start_matches('/'))))
                })
            {
                continue;
            }
        }
        regular(p)?;
    }
    Ok(())
}
pub fn private_dir(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        fs::create_dir(path).map_err(|_| error("MEDIA_PATH"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
    }
    regular(path)?;
    if !path.is_dir() {
        return Err(error("MEDIA_PATH"));
    }
    Ok(())
}
fn open_read(path: &Path, limit: u64) -> Result<File, AppError> {
    if !path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(error("MEDIA_PATH"));
    }
    #[cfg(windows)]
    {
        if path.components().any(|c|matches!(c,std::path::Component::Prefix(p) if !matches!(p.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))) {return Err(error("MEDIA_PATH"))}
    }

    check_ancestors(path)?;
    let before = fs::symlink_metadata(path).map_err(|_| error("MEDIA_PATH"))?;
    if !before.is_file() {
        return Err(error("MEDIA_PATH"));
    }
    if before.len() > limit {
        return Err(error("MEDIA_LIMIT"));
    }
    let mut o = OpenOptions::new();
    o.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        o.custom_flags(0x00200000).share_mode(1);
    }
    let f = o.open(path).map_err(|_| error("MEDIA_PATH"))?;
    let after = f.metadata()?;
    if !after.is_file() || after.len() > limit {
        return Err(error("MEDIA_PATH"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.ino() != after.ino() || before.dev() != after.dev() {
            return Err(error("MEDIA_PATH"));
        }
    }
    check_ancestors(path)?;
    Ok(f)
}
fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, AppError> {
    let f = open_read(path, limit)?;
    let mut bytes = Vec::new();
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error("MEDIA_UNAVAILABLE"))?;
    if bytes.len() as u64 > limit {
        return Err(error("MEDIA_LIMIT"));
    }
    Ok(bytes)
}
fn new_file(path: &Path) -> Result<File, AppError> {
    let mut o = OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path).map_err(|_| error("MEDIA_UNAVAILABLE"))
}
fn sync_dir(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
struct LimitedWriter {
    file: File,
    written: u64,
    limit: u64,
    hash: Sha256,
}
impl Write for LimitedWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() as u64 > self.limit - self.written {
            return Err(std::io::Error::other("media output limit"));
        }
        let n = self.file.write(b)?;
        self.written += n as u64;
        self.hash.update(&b[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
fn encode(image: &DynamicImage, path: &Path, name: &str, limit: u64) -> Result<Variant, AppError> {
    let mut w = LimitedWriter {
        file: new_file(path)?,
        written: 0,
        limit,
        hash: Sha256::new(),
    };
    let rgba = image.as_rgba8().ok_or_else(|| error("MEDIA_INVALID"))?;
    image::codecs::png::PngEncoder::new_with_quality(
        &mut w,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    )
    .write_image(
        rgba.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|_| error("MEDIA_UNAVAILABLE"))?;
    w.flush()?;
    w.file.sync_all()?;
    Ok(Variant {
        name: name.into(),
        bytes: w.written,
        sha256: format!("{:x}", w.hash.finalize()),
        width: image.width(),
        height: image.height(),
    })
}
fn decode(bytes: &[u8]) -> Result<DynamicImage, AppError> {
    let format = image::guess_format(bytes).map_err(|_| error("MEDIA_UNSUPPORTED"))?;
    match format {
        ImageFormat::Jpeg => {
            if !bytes.ends_with(&[0xff, 0xd9]) {
                return Err(error("MEDIA_INVALID"));
            }
        }
        ImageFormat::Png => {
            if !bytes.ends_with(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82]) {
                return Err(error("MEDIA_INVALID"));
            }
            let mut at = 8usize;
            while at < bytes.len() {
                if bytes.len() - at < 12 {
                    return Err(error("MEDIA_INVALID"));
                }
                let size = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
                let end = at
                    .checked_add(size)
                    .and_then(|v| v.checked_add(12))
                    .filter(|v| *v <= bytes.len())
                    .ok_or_else(|| error("MEDIA_INVALID"))?;
                if &bytes[at + 4..at + 8] == b"acTL" {
                    return Err(error("MEDIA_UNSUPPORTED"));
                }
                at = end;
            }
        }
        ImageFormat::WebP => {
            if bytes.len() < 12
                || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize + 8 != bytes.len()
            {
                return Err(error("MEDIA_INVALID"));
            }
            let mut at = 12usize;
            while at < bytes.len() {
                if bytes.len() - at < 8 {
                    return Err(error("MEDIA_INVALID"));
                }
                let tag = &bytes[at..at + 4];
                let size = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
                if tag == b"ANIM" || tag == b"ANMF" {
                    return Err(error("MEDIA_UNSUPPORTED"));
                }
                at = at
                    .checked_add(8)
                    .and_then(|v| v.checked_add(size))
                    .and_then(|v| v.checked_add(size % 2))
                    .filter(|v| *v <= bytes.len())
                    .ok_or_else(|| error("MEDIA_INVALID"))?;
            }
        }
        _ => return Err(error("MEDIA_UNSUPPORTED")),
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(128 * 1024 * 1024);
    limits.max_image_width = Some(30_000_000);
    limits.max_image_height = Some(30_000_000);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(decode_error)?;
    let (w, h) = decoder.dimensions();
    if w == 0
        || h == 0
        || u64::from(w) * u64::from(h) > PIXELS
        || decoder.total_bytes() > 128 * 1024 * 1024
    {
        return Err(error("MEDIA_LIMIT"));
    }
    let orientation = decoder.orientation().map_err(decode_error)?;
    let decoded = DynamicImage::from_decoder(decoder).map_err(decode_error)?;
    // Normalize into a metadata-free, 8-bit RGBA PNG. ICC/high-bit-depth source
    // color precision is not retained; no unsupported color-fidelity promise.
    let mut image = DynamicImage::ImageRgba8(decoded.into_rgba8());
    image.apply_orientation(orientation);
    Ok(image)
}
impl MediaStore {
    pub fn new(directory: &Path) -> Result<Self, AppError> {
        check_ancestors(directory)?;
        let root = directory.join("media-v1");
        private_dir(&root)?;
        private_dir(&root.join("staging"))?;
        private_dir(&root.join("assets"))?;
        private_dir(&root.join("cache"))?;
        Ok(Self { root })
    }
    fn directory(&self, id: &str, staging: bool) -> Result<PathBuf, AppError> {
        valid_id(id)?;
        check_ancestors(&self.root)?;
        let p = self
            .root
            .join(if staging { "staging" } else { "assets" })
            .join(id);
        check_ancestors(p.parent().unwrap())?;
        if p.exists() {
            regular(&p)?;
            if !p.is_dir() {
                return Err(error("MEDIA_PATH"));
            }
        }
        Ok(p)
    }
    pub fn prepare(&self, id: &str, source: &Path) -> Result<Manifest, AppError> {
        let bytes = read_limited(source, INPUT_BYTES)?;
        let image = decode(&bytes)?;
        drop(bytes);
        let dir = self.directory(id, true)?;
        private_dir(&dir)?;
        let original = encode(&image, &dir.join("original.png"), "original", FULL_BYTES)?;
        let preview = image.thumbnail(1280, 1280);
        let preview = encode(&preview, &dir.join("preview.png"), "preview", PREVIEW_BYTES)?;
        let thumb = image.thumbnail(384, 384);
        let thumb = encode(&thumb, &dir.join("thumb.png"), "thumb", THUMB_BYTES)?;
        let manifest = Manifest {
            format_version: 1,
            asset_id: id.into(),
            width: image.width(),
            height: image.height(),
            variants: vec![original, preview, thumb],
        };
        manifest.validate()?;
        let mut f = new_file(&dir.join("manifest.json"))?;
        f.write_all(&serde_json::to_vec(&manifest).map_err(|_| error("MEDIA_UNAVAILABLE"))?)?;
        f.sync_all()?;
        sync_dir(&dir)?;
        Ok(manifest)
    }
    pub fn finalize(&self, manifest: &Manifest) -> Result<(), AppError> {
        self.verify(manifest, true)?;
        let staged = self.directory(&manifest.asset_id, true)?;
        let target = self.directory(&manifest.asset_id, false)?;
        if target.exists() {
            return Err(error("MEDIA_PATH"));
        }
        fs::rename(&staged, &target).map_err(|_| error("MEDIA_UNAVAILABLE"))?;
        sync_dir(target.parent().unwrap())?;
        sync_dir(staged.parent().unwrap())?;
        Ok(())
    }
    pub fn recover(&self, id: &str, expected: Option<&Manifest>) -> Result<Manifest, AppError> {
        let final_dir = self.directory(id, false)?;
        let staged = !final_dir.exists();
        let dir = self.directory(id, staged)?;
        let m: Manifest = serde_json::from_slice(&read_limited(&dir.join("manifest.json"), 8192)?)
            .map_err(|_| error("MEDIA_UNAVAILABLE"))?;
        if m.asset_id != id || expected.is_some_and(|e| e != &m) {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        self.verify(&m, staged)?;
        if staged {
            self.finalize(&m)?
        }
        Ok(m)
    }
    fn verify_file(&self, path: &Path, v: &Variant) -> Result<(), AppError> {
        let file = open_read(path, v.bytes)?;
        let mut hash = Sha256::new();
        let mut n = 0;
        let mut b = [0; 65536];
        let mut reader = file.take(v.bytes + 1);
        loop {
            let len = reader.read(&mut b)?;
            if len == 0 {
                break;
            }
            n += len as u64;
            hash.update(&b[..len]);
        }
        if n != v.bytes || format!("{:x}", hash.finalize()) != v.sha256 {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        Ok(())
    }
    fn recover_ready_directory(&self, m: &Manifest) -> Result<(), AppError> {
        let final_dir = self.directory(&m.asset_id, false)?;
        if !final_dir.exists() && self.directory(&m.asset_id, true)?.exists() {
            self.recover(&m.asset_id, Some(m))?;
        }
        Ok(())
    }
    pub fn verify(&self, m: &Manifest, staged: bool) -> Result<(), AppError> {
        m.validate()?;
        if !staged {
            self.recover_ready_directory(m)?;
        }
        let dir = self.directory(&m.asset_id, staged)?;
        self.verify_file(&dir.join("original.png"), &m.variants[0])?;
        for v in &m.variants[1..] {
            if staged {
                self.verify_file(&dir.join(format!("{}.png", v.name)), v)?;
            } else {
                self.serve(m, &v.name)?;
            }
        }
        Ok(())
    }
    fn checked_variant(&self, path: &Path, v: &Variant) -> Result<Vec<u8>, AppError> {
        let bytes = read_limited(path, v.bytes)?;
        if bytes.len() as u64 != v.bytes
            || format!("{:x}", Sha256::digest(&bytes)) != v.sha256
            || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        Ok(bytes)
    }
    pub fn serve(&self, m: &Manifest, variant: &str) -> Result<Vec<u8>, AppError> {
        m.validate()?;
        if !["thumb", "preview"].contains(&variant) {
            return Err(error("MEDIA_PATH"));
        }
        self.recover_ready_directory(m)?;
        let v = m
            .variants
            .iter()
            .find(|v| v.name == variant)
            .ok_or_else(|| error("MEDIA_UNAVAILABLE"))?;
        let owned = self
            .directory(&m.asset_id, false)?
            .join(format!("{variant}.png"));
        // Do not heal through symlinks/reparse points or hostile directory entries.
        if std::fs::symlink_metadata(&owned).is_ok() {
            regular(&owned)?;
            if !owned.is_file() {
                return Err(error("MEDIA_PATH"));
            }
        }
        if let Ok(bytes) = self.checked_variant(&owned, v) {
            return Ok(bytes);
        }
        let cache = self.root.join("cache");
        check_ancestors(&cache)?;
        let cache = cache.join(&m.asset_id);
        private_dir(&cache)?;
        let target = cache.join(format!("{variant}.png"));
        if std::fs::symlink_metadata(&target).is_ok() {
            regular(&target)?;
            if !target.is_file() {
                return Err(error("MEDIA_PATH"));
            }
        }
        if let Ok(bytes) = self.checked_variant(&target, v) {
            return Ok(bytes);
        }
        let original = self.directory(&m.asset_id, false)?.join("original.png");
        self.verify_file(&original, &m.variants[0])?;
        let mut reader = ImageReader::with_format(
            std::io::BufReader::new(open_read(&original, FULL_BYTES)?),
            ImageFormat::Png,
        );
        let mut limits = image::Limits::default();
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().map_err(decode_error)?;
        if (image.width(), image.height()) != (m.width, m.height) {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        let edge = if variant == "thumb" { 384 } else { 1280 };
        let image = image.thumbnail(edge, edge);
        let pending = cache.join(format!("{variant}.pending.png"));
        // Cache names are not persistent asset keys. Only these reproducible,
        // unreferenced cache files may be replaced; finalized originals stay intact.
        if std::fs::symlink_metadata(&pending).is_ok() {
            regular(&pending)?;
            if !pending.is_file() {
                return Err(error("MEDIA_PATH"));
            }
            fs::remove_file(&pending)?;
        }
        let regenerated = encode(&image, &pending, variant, v.bytes)?;
        if &regenerated != v {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        if target.exists() {
            fs::remove_file(&target)?;
        }
        fs::rename(&pending, &target)?;
        sync_dir(&cache)?;
        self.checked_variant(&target, v)
    }
    // Only known temporary files in a failed, never-referenceable UUID staging
    // directory may be removed. Finalized asset directories are never deleted.
    pub fn clean_staging(&self, id: &str) -> Result<(), AppError> {
        let dir = self.directory(id, true)?;
        if !dir.exists() {
            return Ok(());
        }
        for name in ["original.png", "preview.png", "thumb.png", "manifest.json"] {
            let p = dir.join(name);
            if p.exists() {
                regular(&p)?;
                if !p.is_file() {
                    return Err(error("MEDIA_PATH"));
                }
                fs::remove_file(p)?;
            }
        }
        fs::remove_dir(dir)?;
        Ok(())
    }
}
pub fn request_key(uri: &tauri::http::Uri) -> Result<(String, String), AppError> {
    if uri.query().is_some() || !matches!(uri.host(), Some("localhost" | "recipe-media.localhost"))
    {
        return Err(error("MEDIA_PATH"));
    }
    let parts: Vec<_> = uri.path().trim_start_matches('/').split('/').collect();
    if parts.len() != 2 || !["thumb", "preview"].contains(&parts[1]) {
        return Err(error("MEDIA_PATH"));
    }
    valid_id(parts[0])?;
    Ok((parts[0].into(), parts[1].into()))
}
