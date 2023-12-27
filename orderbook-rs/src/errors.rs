use thiserror::Error;
use url::ParseError;
use ethers::contract::ContractError;
use ethers::providers::ProviderError;
use ethers::providers::{Http, Provider};
use rustc_hex::FromHexError;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::signers::Ledger;


/// RainOrderbookError
/// Enum representing errors thrown by the crate
#[derive(Error, Debug)] 
pub enum RainOrderbookError{
    #[error("Invalid RPC URL")]
    InvalidRPC{
        #[from]
        source: ParseError
    },
    #[error("Invalid Contract Function Call")]
    InvalidContractFunctionCall{
        #[from]
        source: ContractError<Provider<Http>>
    },
    #[error("Invalid Address")]
    InvalidAddress{ 
        #[from]
        source: FromHexError
    },
    #[error("Failed to confirm transaction")]
    TransactionConfirmationError{ 
        #[from]
        source: ProviderError
    },
    #[error("Error in Transaction")]
    TransactionError{
        #[from]
        source:  SignerMiddlewareError<Provider<Http>, Ledger>
    },
    #[error("Failed to fetch Transaction Receipt")]
    TransactionReceiptError,

}