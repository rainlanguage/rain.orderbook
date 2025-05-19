use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, clap::ValueEnum, Clone, PartialEq)]
pub enum SupportedOutputEncoding {
    Binary,
    Hex,
}

// Helper function to handle encoding and writing data
fn write_encoded_data(
    writer: &mut impl Write,
    encoding: SupportedOutputEncoding,
    data: &[u8],
) -> anyhow::Result<()> {
    let hex_encoded_owned: String;
    let bytes_to_write: &[u8] = match encoding {
        SupportedOutputEncoding::Binary => data,
        SupportedOutputEncoding::Hex => {
            hex_encoded_owned = alloy::primitives::hex::encode_prefixed(data);
            hex_encoded_owned.as_bytes()
        }
    };
    writer.write_all(bytes_to_write)?;
    writer.flush()?;
    Ok(())
}

pub fn output(
    output_path: &Option<PathBuf>,
    output_encoding: SupportedOutputEncoding,
    bytes: &[u8],
) -> anyhow::Result<()> {
    if let Some(path) = output_path {
        let mut file = std::fs::File::create(path)?;
        write_encoded_data(&mut file, output_encoding, bytes)
    } else {
        let mut stdout_handle = std::io::stdout();
        write_encoded_data(&mut stdout_handle, output_encoding, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_output_to_file_binary() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = Some(temp_file.path().to_path_buf());
        let data = b"hello world";

        let res = output(&output_path, SupportedOutputEncoding::Binary, data);
        assert!(res.is_ok());

        let mut file_content = Vec::new();
        fs::File::open(temp_file.path())
            .unwrap()
            .read_to_end(&mut file_content)
            .unwrap();
        assert_eq!(file_content, data);
    }

    #[test]
    fn test_output_to_file_hex() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = Some(temp_file.path().to_path_buf());
        let data = b"hello world";
        let expected_hex = alloy::primitives::hex::encode_prefixed(data);

        let res = output(&output_path, SupportedOutputEncoding::Hex, data);
        assert!(res.is_ok());

        let file_content_str = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(file_content_str, expected_hex);
    }

    #[test]
    fn test_write_to_buffer_binary() {
        let data = b"hello world";
        let mut buffer: Vec<u8> = Vec::new();

        let res = write_encoded_data(&mut buffer, SupportedOutputEncoding::Binary, data);
        assert!(res.is_ok(), "write_encoded_data failed: {:?}", res.err());

        assert_eq!(buffer, data, "Buffer content does not match expected data");
    }

    #[test]
    fn test_write_to_buffer_hex() {
        let data = b"hello world";
        let expected_hex = alloy::primitives::hex::encode_prefixed(data);
        let mut buffer: Vec<u8> = Vec::new();

        let res = write_encoded_data(&mut buffer, SupportedOutputEncoding::Hex, data);
        assert!(res.is_ok(), "write_encoded_data failed: {:?}", res.err());

        let captured_output_str =
            String::from_utf8(buffer).expect("Captured output is not valid UTF-8");

        assert_eq!(
            captured_output_str, expected_hex,
            "Captured buffer hex output does not match expected hex string"
        );
    }

    #[test]
    fn test_output_to_file_binary_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = Some(temp_file.path().to_path_buf());
        let data = b"";

        let res = output(&output_path, SupportedOutputEncoding::Binary, data);
        assert!(res.is_ok());

        let mut file_content = Vec::new();
        fs::File::open(temp_file.path())
            .unwrap()
            .read_to_end(&mut file_content)
            .unwrap();
        assert_eq!(file_content, data);
    }

    #[test]
    fn test_output_to_file_hex_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = Some(temp_file.path().to_path_buf());
        let data = b"";
        let expected_hex = alloy::primitives::hex::encode_prefixed(data);

        let res = output(&output_path, SupportedOutputEncoding::Hex, data);
        assert!(res.is_ok());

        let file_content_str = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(file_content_str, expected_hex);
    }

    #[test]
    fn test_write_to_buffer_binary_empty() {
        let data = b"";
        let mut buffer: Vec<u8> = Vec::new();
        let res = write_encoded_data(&mut buffer, SupportedOutputEncoding::Binary, data);
        assert!(
            res.is_ok(),
            "write_encoded_data failed for empty binary: {:?}",
            res.err()
        );
        assert!(
            buffer.is_empty(),
            "Buffer should be empty for binary empty case"
        );
    }

    #[test]
    fn test_write_to_buffer_hex_empty() {
        let data = b"";
        let expected_hex = alloy::primitives::hex::encode_prefixed(data);
        let mut buffer: Vec<u8> = Vec::new();
        let res = write_encoded_data(&mut buffer, SupportedOutputEncoding::Hex, data);
        assert!(
            res.is_ok(),
            "write_encoded_data failed for empty hex: {:?}",
            res.err()
        );
        let captured_output_str =
            String::from_utf8(buffer).expect("Captured output is not valid UTF-8");
        assert_eq!(
            captured_output_str, expected_hex,
            "Buffer content for empty hex data should be '0x'"
        );
    }

    #[test]
    fn test_output_to_file_invalid_path() {
        let output_path = Some(PathBuf::from("/non_existent_directory/test_file.txt"));
        let data = b"hello world";

        let res = output(&output_path, SupportedOutputEncoding::Binary, data);
        assert!(
            res.is_err(),
            "Expected an error for invalid path, but got Ok"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_output_to_file_permission_denied() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::Builder::new()
            .prefix("readonly_dir")
            .tempdir()
            .unwrap();
        let dir_path = temp_dir.path();

        let mut perms = fs::metadata(dir_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(dir_path, Permissions::from_mode(0o555)).unwrap();

        let file_path = dir_path.join("test_file.txt");
        let output_path = Some(file_path);
        let data = b"hello world";

        let res = output(&output_path, SupportedOutputEncoding::Binary, data);
        assert!(res.is_err(), "Expected a permission error, but got Ok");

        fs::set_permissions(dir_path, Permissions::from_mode(0o755)).unwrap();
    }
}
