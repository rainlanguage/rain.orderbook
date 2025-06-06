use alloy::{
    contract::CallBuilder,
    network::{AnyNetwork, AnyReceiptEnvelope, EthereumWallet},
    node_bindings::{Anvil, AnvilInstance},
    primitives::{utils::parse_units, Address, Bytes, U256},
    providers::{
        fillers::{FillProvider, JoinFill, RecommendedFiller, WalletFiller},
        PendingTransactionError, Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolCall,
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use serde_json::value::RawValue;

sol!(
    #![sol(all_derives = true, rpc = true)]
    ERC20, "../../lib/rain.interpreter/out/TestERC20.sol/TestERC20.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Interpreter,
    "../../out/RainterpreterNPE2.sol/RainterpreterNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Store,
    "../../out/RainterpreterStoreNPE2.sol/RainterpreterStoreNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Parser,
    "../../out/RainterpreterParserNPE2.sol/RainterpreterParserNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Deployer,
    "../../out/RainterpreterExpressionDeployerNPE2.sol/RainterpreterExpressionDeployerNPE2.json"
);

// type aliases for LocalEvm provider type
pub type LocalEvmFillers = JoinFill<RecommendedFiller, WalletFiller<EthereumWallet>>;
pub type LocalEvmProvider = FillProvider<LocalEvmFillers, RootProvider<AnyNetwork>, AnyNetwork>;

/// A local evm instance that wraps an Anvil instance and provider with
/// its signers, and with rain contracts already deployed on it.
/// The first signer wallet is the main wallet that would sign any transactions
/// that dont specify a sender ('to' field)
pub struct LocalEvm {
    /// The Anvil instance, ie the local blockchain
    pub anvil: AnvilInstance,

    /// The alloy provider instance of this local blockchain
    pub provider: LocalEvmProvider,

    /// Alloy interpreter contract instance deployed on this blockchain
    pub interpreter: Interpreter::InterpreterInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy store contract instance deployed on this blockchain
    pub store: Store::StoreInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy parser contract instance deployed on this blockchain
    pub parser: Parser::ParserInstance<Http<Client>, LocalEvmProvider>,

    /// Alloy expression deployer contract instance deployed on this blockchain
    pub deployer: Deployer::DeployerInstance<Http<Client>, LocalEvmProvider>,

    /// Array of alloy ERC20 contract instances deployed on this blockchain
    pub tokens: Vec<ERC20::ERC20Instance<LocalEvmProvider, AnyNetwork>>,

    /// All wallets of this local blockchain that can be used to perform transactions
    /// the first wallet is the blockchain's default wallet, ie transactions that dont
    /// explicitly specify a sender address will use this as the sender
    pub signer_wallets: Vec<EthereumWallet>,
}

impl LocalEvm {
    /// Instantiates this struct with rain contracts deployed and no ERC20 tokens
    pub async fn new() -> Self {
        let anvil = Anvil::new().try_spawn().unwrap();

        // set up signers from anvil accounts
        let mut signer_wallets = vec![];
        let mut default_signer =
            EthereumWallet::from(PrivateKeySigner::from(anvil.keys()[0].clone()));
        let other_signer_wallets: Vec<EthereumWallet> = anvil.keys()[1..]
            .iter()
            .map(|v| EthereumWallet::from(PrivateKeySigner::from(v.clone())))
            .collect();

        for s in &other_signer_wallets {
            default_signer.register_signer(s.default_signer())
        }
        signer_wallets.push(default_signer);
        signer_wallets.extend(other_signer_wallets);

        // Create a provider with the wallet and fillers
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(signer_wallets[0].clone())
            .on_http(anvil.endpoint_url());

        // provider.wallet_mut().register_signer(signer)

        // deploy rain contracts
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
            interpreter,
            store,
            parser,
            deployer,
            tokens: vec![],
            signer_wallets,
        }
    }

    /// Instantiates with number of ERC20 tokens with 18 decimals.
    /// Each token after being deployed will mint 1 milion tokens to the
    /// default address, which is the first signer wallet of this instance
    pub async fn new_with_tokens(tokens_count: u8) -> Self {
        let mut local_evm = Self::new().await;

        // deploy tokens contracts and mint 1 milion of each for the default address (first signer wallet)
        for i in 1..=tokens_count {
            local_evm
                .deploy_new_token(
                    &format!("Token{}", i),
                    &format!("Token{}", i),
                    18,
                    parse_units("1_000_000", 18).unwrap().into(),
                    local_evm.anvil.addresses()[0],
                )
                .await;
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
        decimals: u8,
        supply: U256,
        recipient: Address,
    ) -> ERC20::ERC20Instance<LocalEvmProvider, AnyNetwork> {
        let token = ERC20::deploy(
            self.provider.clone(),
            name.to_string(),
            symbol.to_string(),
            decimals,
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
        contract_call: CallBuilder<Http<Client>, &LocalEvmProvider>,
    ) -> Result<TransactionReceipt, PendingTransactionError> {
        self.provider
            .send_transaction(contract_call.into_transaction_request().into())
            .await?
            .get_receipt()
            .await
    }

    /// Sends (write call) a raw transaction request to the blockchain and returns the tx receipt
    pub async fn send_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<
        TransactionReceipt<AnyReceiptEnvelope<alloy::rpc::types::Log>>,
        PendingTransactionError,
    > {
        self.provider
            .send_transaction(tx.into())
            .await?
            .get_receipt()
            .await
            .map(|r| r.inner)
    }

    /// Calls (readonly call) contract method and returns the decoded result
    pub async fn call_contract<T: SolCall>(
        &self,
        contract_call: CallBuilder<Http<Client>, &LocalEvmProvider, AnyNetwork>,
    ) -> Result<
        Result<T::Return, alloy::sol_types::Error>,
        RpcError<TransportErrorKind, Box<RawValue>>,
    > {
        Ok(T::abi_decode_returns(
            &self
                .provider
                .call(contract_call.into_transaction_request())
                .await?,
        ))
    }

    /// Calls (readonly call) a raw transaction and returns the result
    pub async fn call(
        &self,
        tx: TransactionRequest,
    ) -> Result<Bytes, RpcError<TransportErrorKind, Box<RawValue>>> {
        self.provider.call(tx.into()).await
    }
}
