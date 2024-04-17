use alloy_dyn_abi::SolType;
use alloy_ethers_typecast::transaction::{
    ReadContractParametersBuilder, ReadContractParametersBuilderError, ReadableClient,
    ReadableClientError,
};
use alloy_primitives::{
    hex::{decode, FromHexError},
    Address,
};
use alloy_sol_types::sol;
use rain_metaboard_subgraph::metaboard_client::MetaboardSubgraphClient;
use rain_metadata::{KnownMagic, RainMetaDocumentV1Item};
use rain_metadata_bindings::IDescribedByMetaV1;
use thiserror::Error;

pub struct Words {
    metaboard_url: String,
    rpc_url: String,
}

sol!(
    struct AuthoringMetaV2 {
        // `word` is referenced directly in assembly so don't move the field. It MUST
        // be the first item.
        bytes32 word;
        string description;
    }
);

type AuthoringMetas = sol! { AuthoringMetaV2[] };

#[derive(Error, Debug)]
pub enum WordsError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    ReadContractParametersError(#[from] ReadContractParametersBuilderError),
    #[error(transparent)]
    MetaboardSubgraphError(
        #[from] rain_metaboard_subgraph::metaboard_client::MetaboardSubgraphClientError,
    ),
    #[error("Meta bytes do not start with RainMetaDocumentV1 Magic")]
    MetaMagicNumberMismatch,
    #[error("Metadata error {0}")]
    MetadataError(#[from] rain_metadata::Error),
    #[error(transparent)]
    AbiDecodeError(#[from] alloy_sol_types::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl Words {
    pub fn new(metaboard_url: String, rpc_url: String) -> Self {
        Self {
            metaboard_url,
            rpc_url,
        }
    }

    pub async fn get_words_for_contract(
        &self,
        contract_address: Address,
    ) -> Result<(), WordsError> {
        let subgraph_client = MetaboardSubgraphClient::new(self.metaboard_url.parse()?);

        let client = ReadableClient::new_from_url(self.rpc_url.clone())?;

        let parameters = ReadContractParametersBuilder::default()
            .address(contract_address)
            .call(IDescribedByMetaV1::describedByMetaV1Call {})
            .build()?;

        let metahash = client.read(parameters).await.unwrap()._0;

        let meta = subgraph_client.get_meta_by_hash(&metahash).await?;

        let meta_bytes = decode(&meta[0].meta.0)?;

        if !meta_bytes
            .clone()
            .starts_with(&KnownMagic::RainMetaDocumentV1.to_prefix_bytes())
        {
            return Err(WordsError::MetaMagicNumberMismatch);
        }

        // Decode meta to string
        let meta_bytes_slice = meta_bytes.as_slice();
        let rain_meta_document_item = RainMetaDocumentV1Item::cbor_decode(meta_bytes_slice)?;

        println!("rain_meta_document_item: {:?}", rain_meta_document_item);

        let payload = rain_meta_document_item[0].unpack()?;

        let authoring_meta = AuthoringMetas::abi_decode(&payload, true)?;

        for item in authoring_meta.iter() {
            println!("Word: {}", String::from_utf8(item.word.as_slice().into())?);
            println!("Description: {}", item.description);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_words_for_contract() {
        let metaboard_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/test-polygon/0.0.1/gn".to_string();
        let rpc_url = "https://rpc.ankr.com/polygon".to_string();
        let contract_address = "0xfca89cD12Ba1346b1ac570ed988AB43b812733fe"
            .parse()
            .unwrap();

        let words = Words::new(metaboard_url, rpc_url);

        let result = words.get_words_for_contract(contract_address).await;

        match result {
            Ok(_) => println!("Success"),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
