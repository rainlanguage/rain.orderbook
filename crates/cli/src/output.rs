use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, clap::ValueEnum, Clone, PartialEq)]
pub enum SupportedOutputEncoding {
    Binary,
    Hex,
}

pub fn output(
    output_path: &Option<PathBuf>,
    output_encoding: SupportedOutputEncoding,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let hex_encoded: String;
    let encoded_bytes: &[u8] = match output_encoding {
        SupportedOutputEncoding::Binary => bytes,
        SupportedOutputEncoding::Hex => {
            hex_encoded = alloy::primitives::hex::encode_prefixed(bytes);
            hex_encoded.as_bytes()
        }
    };
    if let Some(output_path) = output_path {
        std::fs::write(output_path, encoded_bytes)?
    } else {
        std::io::stdout().write_all(encoded_bytes)?
    }
    Ok(())
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
    fn test_output_to_stdout_binary() {
        let data = b"hello world";
        // For stdout, we can't easily capture and assert the output in a simple unit test.
        // We will just check if the function runs without error.
        let res = output(&None, SupportedOutputEncoding::Binary, data);
        assert!(res.is_ok());
    }

    #[test]
    fn test_output_to_stdout_hex() {
        let data = b"hello world";
        // Similar to the binary stdout test, we check for successful execution.
        let res = output(&None, SupportedOutputEncoding::Hex, data);
        assert!(res.is_ok());
    }
}
