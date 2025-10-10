//! REVM-backed interpreter host implementation for the Virtual Raindex.

use std::{collections::HashMap, sync::Arc};

use alloy::primitives::{Address, B256, U256};
use alloy::sol_types::SolCall;
use parking_lot::RwLock;
use rain_interpreter_bindings::IInterpreterV4::{eval4Call, EvalV4};
use revm::{
    context::{
        result::{ExecutionResult, Output, SuccessReason},
        Context,
    },
    database::InMemoryDB,
    primitives::{Address as RevmAddress, Bytes as RevmBytes},
    state::AccountInfo,
    MainBuilder, MainContext, SystemCallEvm,
};

use crate::{
    cache::CodeCache,
    state::{Env, StoreKey},
    BytecodeKind, RaindexError, Result,
};

/// Abstraction for running interpreter bytecode.
pub trait InterpreterHost: Send + Sync {
    /// Executes the Rain interpreter `eval4` entrypoint with the provided inputs.
    fn eval4(
        &self,
        interpreter: Address,
        eval: &EvalV4,
        store_snapshot: &HashMap<StoreKey, B256>,
        env: Env,
    ) -> Result<EvalOutcome>;
}

/// Successful outputs captured from an interpreter evaluation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvalOutcome {
    pub stack: Vec<B256>,
    pub writes: Vec<B256>,
}

/// REVM-backed implementation of [`InterpreterHost`].
pub struct RevmInterpreterHost<C>
where
    C: CodeCache,
{
    code_cache: Arc<C>,
    base_db: RwLock<InMemoryDB>,
}

impl<C> RevmInterpreterHost<C>
where
    C: CodeCache,
{
    /// Creates a new host using the provided bytecode cache as its backing source.
    pub fn new(code_cache: Arc<C>) -> Self {
        Self {
            code_cache,
            base_db: RwLock::new(InMemoryDB::default()),
        }
    }
}

impl<C> InterpreterHost for RevmInterpreterHost<C>
where
    C: CodeCache,
{
    /// Executes interpreter bytecode inside REVM, loading artifacts on demand.
    fn eval4(
        &self,
        interpreter: Address,
        eval: &EvalV4,
        store_snapshot: &HashMap<StoreKey, B256>,
        env: Env,
    ) -> Result<EvalOutcome> {
        self.ensure_code_loaded(interpreter, eval.store)?;
        let db = self.base_db.read().clone();
        let mut evm = Context::mainnet().with_db(db).build_mainnet();

        evm.ctx.block.number = U256::from(env.block_number);
        evm.ctx.block.timestamp = U256::from(env.timestamp);

        let mut eval_call = eval.clone();
        eval_call.stateOverlay = build_state_overlay(store_snapshot, eval.store, eval.namespace);

        let calldata = eval4Call { eval: eval_call }.abi_encode();
        let revm_address = to_revm_address(interpreter);
        let output = evm
            .transact_system_call_finalize(revm_address, RevmBytes::from(calldata.clone()))
            .map_err(|err| RaindexError::RevmExecution(err.to_string()))?;

        match output.result {
            ExecutionResult::Success {
                reason: SuccessReason::Return,
                output,
                ..
            } => {
                let bytes = match output {
                    Output::Call(data) => data,
                    other => {
                        return Err(RaindexError::RevmExecution(format!(
                            "unexpected output: {other:?}"
                        )))
                    }
                };
                let returns = eval4Call::abi_decode_returns(bytes.as_ref())
                    .map_err(|err| RaindexError::RevmExecution(err.to_string()))?;
                Ok(EvalOutcome {
                    stack: returns.stack,
                    writes: returns.writes,
                })
            }
            other => Err(RaindexError::RevmExecution(format!(
                "execution failed: {other:?}"
            ))),
        }
    }
}

impl<C> RevmInterpreterHost<C>
where
    C: CodeCache,
{
    /// Ensures interpreter and store bytecode are present in the REVM database.
    fn ensure_code_loaded(&self, interpreter: Address, store: Address) -> Result<()> {
        let mut db = self.base_db.write();
        self.ensure_contract(&mut db, interpreter, BytecodeKind::Interpreter)?;
        self.ensure_contract(&mut db, store, BytecodeKind::Store)
    }

    /// Loads bytecode into the REVM cache if the account is not yet present.
    fn ensure_contract(
        &self,
        db: &mut InMemoryDB,
        address: Address,
        kind: BytecodeKind,
    ) -> Result<()> {
        let revm_address = to_revm_address(address);
        if db.cache.accounts.contains_key(&revm_address) {
            return Ok(());
        }

        let bytecode = match kind {
            BytecodeKind::Interpreter => self.code_cache.interpreter(address),
            BytecodeKind::Store => self.code_cache.store(address),
        }
        .ok_or(RaindexError::MissingBytecode { address, kind })?;

        let info = AccountInfo::default().with_code((*bytecode).clone());
        db.insert_account_info(revm_address, info);
        Ok(())
    }
}

/// Builds the state overlay vector expected by the interpreter for a namespace.
fn build_state_overlay(
    snapshot: &HashMap<StoreKey, B256>,
    store_address: Address,
    namespace: U256,
) -> Vec<B256> {
    let namespace = B256::from(namespace.to_be_bytes());
    let mut overlay = Vec::new();
    for (key, value) in snapshot
        .iter()
        .filter(|(k, _)| k.store == store_address && k.fqn == namespace)
    {
        overlay.push(key.key);
        overlay.push(*value);
    }
    overlay
}

/// Convenience helper for translating `Address` into the REVM representation.
fn to_revm_address(address: Address) -> RevmAddress {
    RevmAddress::from_slice(address.as_slice())
}
