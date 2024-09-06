use super::*;
use alloy::{
    contract::CallBuilder,
    network::{Ethereum, EthereumWallet},
    node_bindings::{Anvil, AnvilInstance},
    primitives::{utils::parse_units, Address, FixedBytes, U256},
    providers::{
        fillers::{FillProvider, JoinFill, RecommendedFiller, WalletFiller},
        Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol_types::SolCall,
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use std::collections::HashMap;
use std::marker::PhantomData;

// type aliases for LocalEvm provider type
pub type LocalEvmFillers = JoinFill<RecommendedFiller, WalletFiller<EthereumWallet>>;
pub type LocalEvmProvider =
    FillProvider<LocalEvmFillers, RootProvider<Http<Client>>, Http<Client>, Ethereum>;

/// A local evm instance that wraps an Anvil instance and provider with
/// its signers, and with rain contract already deployed on it.
/// The first signer wallet is the main wallet that would sign any transactions
/// that dont specify a sender ('to' field)
pub struct LocalEvm {
    pub provider: LocalEvmProvider,
    pub anvil: AnvilInstance,
    pub orderbook: Orderbook::OrderbookInstance<Http<Client>, LocalEvmProvider>,
    pub interpreter: Interpreter::InterpreterInstance<Http<Client>, LocalEvmProvider>,
    pub store: Store::StoreInstance<Http<Client>, LocalEvmProvider>,
    pub parser: Parser::ParserInstance<Http<Client>, LocalEvmProvider>,
    pub deployer: Deployer::DeployerInstance<Http<Client>, LocalEvmProvider>,
    pub tokens: HashMap<Address, ERC20::ERC20Instance<Http<Client>, LocalEvmProvider>>,
    pub signer_wallets: Vec<EthereumWallet>,
}

impl LocalEvm {
    /// Instantiates this struct with rain contracts deployed and no ERC20 tokens
    pub async fn new() -> Self {
        let anvil = Anvil::new().spawn();

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
            interpreter,
            store,
            parser,
            deployer,
            tokens: HashMap::new(),
            signer_wallets: signers,
        }
    }

    /// Instantiates with number of ERC20 tokens
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
            local_evm.tokens.insert(*token.address(), token);
        }
        local_evm
    }

    /// Get the local rpc url the underlying anvil is running on
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
        self.tokens.insert(*token.address(), token.clone());
        token
    }

    /// Sends a contract write call transaction to the evm instance and returns the tx hash
    pub async fn send_contract_call_transaction<T: SolCall>(
        &self,
        contract_call: CallBuilder<Http<Client>, &LocalEvmProvider, PhantomData<T>>,
    ) -> Result<FixedBytes<32>, RpcError<TransportErrorKind>> {
        self.provider
            .send_transaction(contract_call.into_transaction_request())
            .await?
            .watch()
            .await
    }

    /// Sends a raw transaction request to the evm instance and returns the tx hash
    pub async fn send_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<FixedBytes<32>, RpcError<TransportErrorKind>> {
        self.provider.send_transaction(tx).await?.watch().await
    }
}
