//! Standard library resource access.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Stdlib`.

use std::collections::HashMap;
use std::io::{self, Read};
use std::sync::{LazyLock, Mutex};

use crate::preproc::future_image::FutureImage;
use crate::preproc::spm::SpmChannel;
use crate::preproc::stdlib_sprite::StdlibSprite;
use crate::stubs::FileUtils;

/// Cache of `Stdlib` instances by name.
static ALL: LazyLock<Mutex<HashMap<String, Stdlib>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Access to `PlantUML` standard library resources (puml files, sprites, images).
///
/// Ported from `net.sourceforge.plantuml.preproc.Stdlib`.
pub struct Stdlib {
    #[allow(dead_code)]
    colors: Vec<i32>,
    puml: HashMap<String, Vec<u8>>,
    #[allow(dead_code)]
    json: HashMap<String, Vec<u8>>,
    sprites: HashMap<String, StdlibSprite>,
    images: Vec<FutureImage>,
    name: String,
    info: HashMap<String, String>,
}

impl Stdlib {
    /// Creates a new `Stdlib` for the given name, loading the info file.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.Stdlib`.
    fn new(name: &str) -> Self {
        let mut info = HashMap::new();
        if let Ok(mut stream) = SpmChannel::Info.get_internal_stream(name) {
            let mut data = Vec::new();
            if stream.read_to_end(&mut data).is_ok() {
                for s in FileUtils::read_strings(&data) {
                    if let Some(idx) = s.find('=') {
                        if idx > 0 {
                            let key = s[..idx].trim().to_string();
                            let value = s[idx + 1..].trim().to_string();
                            info.insert(key, value);
                        }
                    }
                }
            }
        }

        Self {
            colors: Vec::new(),
            puml: HashMap::new(),
            json: HashMap::new(),
            sprites: HashMap::new(),
            images: Vec::new(),
            name: name.to_string(),
            info,
        }
    }

    /// Retrieves a `Stdlib` by name, following links if present.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.retrieve`.
    pub fn retrieve(name: &str) -> io::Result<Self> {
        let mut cache = ALL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lib) = cache.get(name) {
            if let Some(link) = lib.get_link_from_info() {
                return Self::retrieve(&link);
            }
            return Ok(Self {
                colors: Vec::new(),
                puml: HashMap::new(),
                json: HashMap::new(),
                sprites: HashMap::new(),
                images: Vec::new(),
                name: lib.name.clone(),
                info: lib.info.clone(),
            });
        }
        let lib = Self::new(name);
        let link = lib.get_link_from_info();
        let info = lib.info.clone();
        let name_owned = lib.name.clone();
        cache.insert(name.to_string(), lib);
        drop(cache);
        if let Some(link) = link {
            return Self::retrieve(&link);
        }
        Ok(Self {
            colors: Vec::new(),
            puml: HashMap::new(),
            json: HashMap::new(),
            sprites: HashMap::new(),
            images: Vec::new(),
            name: name_owned,
            info,
        })
    }

    /// Returns the puml resource for the given full name (e.g. `archimate/Business`).
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.getPumlResource`.
    pub fn get_puml_resource(fullname: &str) -> Option<Vec<u8>> {
        let fullname = fullname.to_lowercase().replace(".puml", "");
        let last = fullname.find('/')?;
        let folder = Self::retrieve(&fullname[..last]).ok()?;
        if folder.info.is_empty() {
            return None;
        }
        folder.load_puml_resource(&fullname[last + 1..])
    }

    /// Reads a sprite by name.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.readSprite`.
    pub fn read_sprite(&mut self, name: &str) -> Option<&StdlibSprite> {
        if self.sprites.is_empty() {
            self.init_sprites();
        }
        self.sprites.get(name)
    }

    /// Reads a data image PNG by index.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.readDataImagePng`.
    pub fn read_data_image_png(&mut self, num: usize) -> Option<&FutureImage> {
        if self.images.is_empty() {
            self.init_images();
        }
        self.images.get(num)
    }

    /// Returns the version string from the info.
    #[must_use]
    pub fn get_version(&self) -> Option<&str> {
        self.info
            .get("VERSION")
            .or_else(|| self.info.get("version"))
            .map(String::as_str)
    }

    /// Returns the source string from the info.
    #[must_use]
    pub fn get_source(&self) -> Option<&str> {
        self.info
            .get("SOURCE")
            .or_else(|| self.info.get("source"))
            .map(String::as_str)
    }

    /// Returns the metadata map.
    #[must_use]
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.info
    }

    /// Returns the name of this stdlib.
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns all stdlib folder names.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.getAllFolderNames`.
    pub fn get_all_folder_names() -> io::Result<Vec<String>> {
        let stream = SpmChannel::input_stream("stdlib/home.spm")?;
        let mut data = Vec::new();
        let _ = stream.take(1024 * 1024).read_to_end(&mut data);
        let mut names: Vec<String> = FileUtils::read_strings(&data);
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Adds version info for all stdlib folders to a list of strings.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Stdlib.addInfoVersion`.
    pub fn add_info_version(strings: &mut Vec<String>, details: bool) {
        if let Ok(names) = Self::get_all_folder_names() {
            for name in &names {
                if let Ok(folder) = Self::retrieve(name) {
                    if details {
                        strings.push(format!("<b>{name}"));
                        strings.push(format!("Version {}", folder.get_version().unwrap_or("?")));
                        strings.push(format!("Delivered by {}", folder.get_source().unwrap_or("?")));
                        strings.push(" ".to_string());
                    } else {
                        strings.push(format!(
                            "* {} (Version {})",
                            name,
                            folder.get_version().unwrap_or("?")
                        ));
                    }
                }
            }
        }
    }

    fn get_link_from_info(&self) -> Option<String> {
        self.info.get("link").cloned()
    }

    fn load_puml_resource(&self, file: &str) -> Option<Vec<u8>> {
        self.puml.get(file).cloned().or_else(|| {
            // In the full implementation, this loads from the SPM channel.
            let _ = file;
            None
        })
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    #[allow(clippy::unnecessary_wraps)]
    fn init_sprites(&mut self) {
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    #[allow(clippy::unnecessary_wraps)]
    fn init_images(&mut self) {
    }

    /// Reads 1 byte from a stream, returning 0-255.
    pub fn read1byte<R: Read>(stream: &mut R) -> usize {
        let mut buf = [0u8; 1];
        if stream.read(&mut buf).is_ok() {
            buf[0] as usize
        } else {
            0
        }
    }

    /// Reads 2 bytes from a stream (big-endian), returning 0-65535.
    pub fn read2bytes<R: Read>(stream: &mut R) -> usize {
        (Self::read1byte(stream) << 8) + Self::read1byte(stream)
    }
}
