use std::{fs, io, path::Path};

pub const MAX_INPUT_FILE_BYTES: u64 = 1024 * 1024;

pub fn read_to_string(path: &Path, description: &str) -> io::Result<String> {
    let metadata = fs::metadata(path).map_err(|error| read_error(path, description, error))?;
    if metadata.len() > MAX_INPUT_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{description} `{}` is {} bytes, exceeding the 1 MiB limit",
                path.display(),
                metadata.len()
            ),
        ));
    }

    fs::read_to_string(path).map_err(|error| read_error(path, description, error))
}

fn read_error(path: &Path, description: &str, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("failed to read {description} `{}`: {error}", path.display()),
    )
}
