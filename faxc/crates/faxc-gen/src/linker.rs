use std::path::Path;

pub struct Linker {
    pub target: String,
    pub linker_cmd: String,
}

impl Linker {
    pub fn new(target: String) -> Self {
        let linker_cmd = if target.contains("windows") {
            "link.exe".to_string()
        } else {
            "ld".to_string()
        };
        Self { target, linker_cmd }
    }

    pub fn link(&self, _objects: &[&Path], _output: &Path, _libs: &[String]) -> Result<(), String> {
        Ok(())
    }
}
