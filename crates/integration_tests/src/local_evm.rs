use super::*;
use alloy::{
    contract::CallBuilder,
    network::{Ethereum, EthereumWallet},
    node_bindings::{Anvil, AnvilInstance},
    primitives::{utils::parse_units, Address, Bytes, U256},
    providers::{
        fillers::{FillProvider, JoinFill, RecommendedFiller, WalletFiller},
        Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::local::PrivateKeySigner,
    sol_types::SolCall,
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use serde_json::value::RawValue;
use std::marker::PhantomData;

// type aliases for LocalEvm provider type
pub type LocalEvmFillers = JoinFill<RecommendedFiller, WalletFiller<EthereumWallet>>;
pub type LocalEvmProvider =
    FillProvider<LocalEvmFillers, RootProvider<Http<Client>>, Http<Client>, Ethereum>;

/// A local evm instance that wraps an Anvil instance and provider with
/// its signers, and with rain contracts already deployed on it.
/// The first signer wallet is the main wallet that would sign any transactions
/// that dont specify a sender ('to' field)
pub struct LocalEvm {
    /// The alloy provider instance of this local blockchain
    pub provider: LocalEvmProvider,

    /// The Anvil instance, ie the local blockchain
    pub anvil: AnvilInstance,

    /// Alloy orderbook contract instance deployed on this blockchain
    pub orderbook: Orderbook::OrderbookInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy orderbook subparser contract instance deployed on this blockchain
    pub orderbook_subparser:
        OrderbookSubParser::OrderbookSubParserInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy interpreter contract instance deployed on this blockchain
    pub interpreter: Interpreter::InterpreterInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy store contract instance deployed on this blockchain
    pub store: Store::StoreInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy parser contract instance deployed on this blockchain
    pub parser: Parser::ParserInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy expression deployer contract instance deployed on this blockchain
    pub deployer: Deployer::DeployerInstance<Http<Client>, LocalEvmProvider>,

    /// Array of alloy ERC20 contract instances deployed on this blockchain
    pub tokens: Vec<ERC20::ERC20Instance<Http<Client>, LocalEvmProvider>>,

    /// All wallets of this local blockchain that can be used to perform transactions
    /// the first wallet is the blockchain's default wallet, ie transactions that dont
    /// explicitly specify a sender address will use this as the sender
    pub signer_wallets: Vec<EthereumWallet>,
}

impl LocalEvm {
    /// Instantiates this struct with rain contracts deployed and no ERC20 tokens
    pub async fn new() -> Self {
        let anvil = Anvil::new().try_spawn().unwrap();

        let signers: Vec<EthereumWallet> = anvil
            .keys()
            .iter()
            .map(|v| EthereumWallet::from(PrivateKeySigner::from(v.clone())))
            .collect();

        // Create a provider with the wallet.
        let rpc_url = anvil.endpoint_url();
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(signers[0].clone())
            .on_http(rpc_url);

        // deploy rain contracts
        let orderbook = Orderbook::deploy(provider.clone()).await.unwrap();
        let orderbook_subparser = OrderbookSubParser::deploy(provider.clone()).await.unwrap();
        let interpreter = Interpreter::deploy(provider.clone()).await.unwrap();
        let store = Store::deploy(provider.clone()).await.unwrap();
        let parser = Parser::deploy(provider.clone()).await.unwrap();
        let config = Deployer::RainterpreterExpressionDeployerNPE2ConstructionConfigV2 {
            interpreter: *interpreter.address(),
            parser: *parser.address(),
            store: *store.address(),
        };
        let deployer = Deployer::deploy(provider.clone(), config).await.unwrap();

        Self {
            anvil,
            provider,
            orderbook,
            orderbook_subparser,
            interpreter,
            store,
            parser,
            deployer,
            tokens: vec![],
            signer_wallets: signers,
        }
    }

    /// Instantiates with number of ERC20 tokens with 18 decimals.
    /// Each token after being deployed will mint 1 milion tokens to the
    /// default address, which is the first signer wallet of this instance
    pub async fn new_with_tokens(token_count: u8) -> Self {
        let mut local_evm = Self::new().await;

        // deploy tokens contracts and mint 1 milion of each for the default address (first signer wallet)
        for i in 1..=token_count {
            let token = ERC20::deploy(
                local_evm.provider.clone(),
                format!("Token{}", i),
                format!("Token{}", i),
                local_evm.signer_wallets[0].default_signer().address(),
                parse_units("1000000", 18).unwrap().into(),
            )
            .await
            .unwrap();
            local_evm.tokens.push(token);
        }
        local_evm
    }

    /// Get the local rpc url of the underlying anvil instance
    pub fn url(&self) -> String {
        self.anvil.endpoint()
    }

    /// Deploys a new ERC20 token with the given arguments
    pub async fn deploy_new_token(
        &mut self,
        name: &str,
        symbol: &str,
        recipient: Address,
        supply: U256,
    ) -> ERC20::ERC20Instance<Http<Client>, LocalEvmProvider> {
        let token = ERC20::deploy(
            self.provider.clone(),
            name.to_string(),
            symbol.to_string(),
            recipient,
            supply,
        )
        .await
        .unwrap();
        self.tokens.push(token.clone());
        token
    }

    /// Sends a contract write transaction to the blockchain and returns the tx receipt
    pub async fn send_contract_transaction<T: SolCall>(
        &self,
        contract_call: CallBuilder<Http<Client>, &LocalEvmProvider, PhantomData<T>>,
    ) -> Result<TransactionReceipt, RpcError<TransportErrorKind, Box<RawValue>>> {
        self.provider
            .send_transaction(contract_call.into_transaction_request())
            .await?
            .get_receipt()
            .await
    }

    /// Sends (write call) a raw transaction request to the blockchain and returns the tx receipt
    pub async fn send_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<TransactionReceipt, RpcError<TransportErrorKind, Box<RawValue>>> {
        self.provider
            .send_transaction(tx)
            .await?
            .get_receipt()
            .await
    }

    /// Calls (readonly call) contract method and returns the decoded result
    pub async fn call_contract<T: SolCall>(
        &self,
        contract_call: CallBuilder<Http<Client>, &LocalEvmProvider, PhantomData<T>>,
    ) -> Result<
        Result<T::Return, alloy::sol_types::Error>,
        RpcError<TransportErrorKind, Box<RawValue>>,
    > {
        Ok(T::abi_decode_returns(
            &self
                .provider
                .call(&contract_call.into_transaction_request())
                .await?,
            true,
        ))
    }

    /// Calls (readonly call) a raw transaction and returns the result
    pub async fn call(
        &self,
        tx: &TransactionRequest,
    ) -> Result<Bytes, RpcError<TransportErrorKind, Box<RawValue>>> {
        self.provider.call(tx).await
    }
}
