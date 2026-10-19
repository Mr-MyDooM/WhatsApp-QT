//! Profile pictures of the accounts. The remote WhatsApp page sends its own avatar through
//! the `profile_picture` command. Nothing from the page is trusted: this module checks the
//! data URL, limits the call rate, and stores one small file per account in the app data
//! folder (never in `settings.json`).

use std::collections::HashMap;
use std::path::PathBuf;

use base64::Engine;

const PREFIXES: [(&str, &[u8]); 2] = [
    ("data:image/jpeg;base64,", &[0xFF, 0xD8, 0xFF]),
    ("data:image/png;base64,", &[0x89, b'P', b'N', b'G']),
];
/// Largest decoded picture. The page script sends a 96x96 JPEG of about 5 KB.
const MAX_BYTES: usize = 64 * 1024;
/// Smallest time between two calls of one account. Every call counts, accepted or not.
const MIN_CALL_GAP_SECS: u64 = 10; // Keep below POLL_MS in avatar_shim.js.

/// Accepts only a small JPEG or PNG data URL whose bytes decode and start with the right
/// magic number. Returns the same string, so it can be stored as it is.
pub fn validate(data_url: &str) -> Result<&str, &'static str> {
    let (prefix, magic) = PREFIXES
        .iter()
        .find(|(p, _)| data_url.starts_with(p))
        .ok_or("not a jpeg or png data URL")?;
    let b64 = &data_url[prefix.len()..];
    // Base64 has 4 characters for 3 bytes. Reject before decoding a huge string.
    if b64.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err("picture too large");
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| "bad base64")?;
    if bytes.len() > MAX_BYTES {
        return Err("picture too large");
    }
    if !bytes.starts_with(magic) {
        return Err("bytes do not match the image type");
    }
    Ok(data_url)
}

/// Pictures in memory, with their files and the call rate limit.
pub struct Avatars {
    dir: PathBuf,
    pictures: HashMap<String, String>,
    last_call: HashMap<String, u64>,
}

impl Avatars {
    /// Reads every valid `<uuid>.txt` in `dir`. A bad file is skipped.
    pub fn load(dir: PathBuf) -> Self {
        let mut pictures = HashMap::new();
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let is_txt = path.extension().is_some_and(|e| e == "txt");
            let Some(id) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| is_txt && uuid::Uuid::parse_str(s).is_ok())
            else {
                continue;
            };
            if let Ok(text) = std::fs::read_to_string(&path) {
                if validate(&text).is_ok() {
                    pictures.insert(id.to_string(), text);
                }
            }
        }
        Self {
            dir,
            pictures,
            last_call: HashMap::new(),
        }
    }

    pub fn all(&self) -> &HashMap<String, String> {
        &self.pictures
    }

    fn path(&self, id: &str) -> Option<PathBuf> {
        uuid::Uuid::parse_str(id).ok()?;
        Some(self.dir.join(format!("{id}.txt")))
    }

    /// Stores a picture. Returns `Ok(true)` when the picture changed, `Ok(false)` when the
    /// call is rate limited or the picture is the same. The caller checks that `id` is a
    /// known account.
    pub fn submit(&mut self, id: &str, data_url: &str, now: u64) -> Result<bool, String> {
        if let Some(last) = self.last_call.get(id) {
            if now.saturating_sub(*last) < MIN_CALL_GAP_SECS {
                return Ok(false);
            }
        }
        self.last_call.insert(id.to_string(), now);
        let url = validate(data_url).map_err(str::to_string)?;
        if self.pictures.get(id).is_some_and(|old| old == url) {
            return Ok(false);
        }
        let path = self.path(id).ok_or("bad account id")?;
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        std::fs::write(&path, url).map_err(|e| e.to_string())?;
        self.pictures.insert(id.to_string(), url.to_string());
        Ok(true)
    }

    /// Deletes the picture of a removed account.
    pub fn remove(&mut self, id: &str) {
        self.pictures.remove(id);
        self.last_call.remove(id);
        if let Some(path) = self.path(id) {
            if let Err(e) = std::fs::remove_file(&path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::warn!("Could not delete {}: {e}", path.display());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "11111111-2222-3333-4444-555555555555";

    fn jpeg(len: usize) -> String {
        let mut bytes = vec![0xFF, 0xD8, 0xFF];
        bytes.resize(len, 0);
        format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        )
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wa-avatar-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn accepts_small_jpeg_and_png() {
        assert!(validate(&jpeg(100)).is_ok());
        let png = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G', 1])
        );
        assert!(validate(&png).is_ok());
    }

    #[test]
    fn refuses_picture_over_cap() {
        assert!(validate(&jpeg(MAX_BYTES)).is_ok());
        assert!(validate(&jpeg(MAX_BYTES + 1)).is_err());
    }

    #[test]
    fn refuses_wrong_type_and_bad_bytes() {
        assert!(validate("data:image/svg+xml;base64,AAAA").is_err());
        assert!(validate("data:text/html;base64,AAAA").is_err());
        assert!(validate("https://example.com/a.jpg").is_err());
        assert!(validate("data:image/jpeg;base64,!!!!").is_err());
        // PNG label, JPEG bytes.
        let mixed = jpeg(10).replace("image/jpeg", "image/png");
        assert!(validate(&mixed).is_err());
    }

    #[test]
    fn stores_reloads_and_removal_deletes_the_file() {
        let dir = temp_dir("store");
        let mut a = Avatars::load(dir.clone());
        assert_eq!(a.submit(ID, &jpeg(50), 100), Ok(true));
        assert!(dir.join(format!("{ID}.txt")).exists());
        assert!(Avatars::load(dir.clone()).all().contains_key(ID));
        a.remove(ID);
        assert!(!dir.join(format!("{ID}.txt")).exists());
        assert!(a.all().is_empty());
        assert!(Avatars::load(dir.clone()).all().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rate_limits_per_account_and_keeps_old_picture_on_refusal() {
        let dir = temp_dir("rate");
        let mut a = Avatars::load(dir.clone());
        assert_eq!(a.submit(ID, &jpeg(50), 100), Ok(true));
        assert_eq!(a.submit(ID, &jpeg(60), 101), Ok(false));
        assert_eq!(a.all()[ID], jpeg(50));
        assert!(a.submit(ID, "data:text/html;base64,AAAA", 200).is_err());
        assert_eq!(a.all()[ID], jpeg(50));
        assert_eq!(a.submit(ID, &jpeg(60), 300), Ok(true));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn refuses_id_that_is_not_a_uuid() {
        let dir = temp_dir("id");
        let mut a = Avatars::load(dir.clone());
        assert!(a.submit("../evil", &jpeg(50), 100).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
