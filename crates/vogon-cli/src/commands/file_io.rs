use std::{fs, io, path::Path};

pub fn read_to_string(path: &Path, description: &str) -> io::Result<String> {
    fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {description} `{}`: {error}", path.display()),
        )
    })
}
