use strum::EnumIter;
use strum::EnumString;

#[derive(serde::Serialize, Clone, Copy, EnumString, EnumIter, strum::Display, Debug, PartialEq)]
#[strum(serialize_all = "kebab_case")]
#[serde(rename_all = "kebab-case")]
#[repr(u64)]
pub enum KnownMagic {
    RainMetaDocumentV1 = 0xff0a89c674ee7874,
    SolidityAbiV2 = 0xffe5ffb4a3ff2cde,
    OpMetaV1 = 0xffe5282f43e495b4,
    InterpreterCallerMetaV1 = 0xffc21bbf86cc199b,
}

impl KnownMagic {
    pub fn to_prefix_bytes(&self) -> [u8; 8] {
        // Use big endian here as the magic numbers are for binary data prefixes.
        (*self as u64).to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::KnownMagic;

    #[test]
    fn test_rain_meta_document_v1() {
        let magic_number = KnownMagic::RainMetaDocumentV1;
        let magic_number_after_prefix = magic_number.to_prefix_bytes();

        assert_eq!(hex::encode(magic_number_after_prefix), "ff0a89c674ee7874");
    }

    #[test]
    fn test_solidity_abi_v2() {
        let magic_number = KnownMagic::SolidityAbiV2;
        let magic_number_after_prefix = magic_number.to_prefix_bytes();

        assert_eq!(hex::encode(magic_number_after_prefix), "ffe5ffb4a3ff2cde");
    }

    #[test]
    fn test_op_meta_v1() {
        let magic_number = KnownMagic::OpMetaV1;
        let magic_number_after_prefix = magic_number.to_prefix_bytes();

        assert_eq!(hex::encode(magic_number_after_prefix), "ffe5282f43e495b4");
    }

    #[test]
    fn test_interpreter_caller_meta_v1() {
        let magic_number = KnownMagic::InterpreterCallerMetaV1;
        let magic_number_after_prefix = magic_number.to_prefix_bytes();

        assert_eq!(hex::encode(magic_number_after_prefix), "ffc21bbf86cc199b");
    }
}
