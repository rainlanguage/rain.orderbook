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
    store::build_state_overlay,
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

/// Convenience helper for translating `Address` into the REVM representation.
fn to_revm_address(address: Address) -> RevmAddress {
    RevmAddress::from_slice(address.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::StaticCodeCache;
    use std::sync::Arc;

    fn make_address(byte: u8) -> Address {
        Address::from([byte; 20])
    }

    #[test]
    fn ensure_contract_inserts_bytecode_when_missing() {
        let cache = Arc::new(StaticCodeCache::default());
        let host = RevmInterpreterHost::new(cache.clone());
        let interpreter = make_address(0x33);
        let bytecode = [0x60, 0x00, 0x60, 0x00, 0x01];

        cache.upsert_interpreter(interpreter, &bytecode);

        let mut db = InMemoryDB::default();
        host.ensure_contract(&mut db, interpreter, BytecodeKind::Interpreter)
            .expect("bytecode must be cached");

        let revm_address = to_revm_address(interpreter);
        let account = db
            .cache
            .accounts
            .get(&revm_address)
            .expect("account should be present");
        let info = account.info().expect("account info should exist");
        let stored_code = info.code.expect("stored code should be present");
        let expected: &[u8] = &bytecode;
        assert_eq!(stored_code.original_byte_slice(), expected);
    }

    #[test]
    fn ensure_contract_missing_bytecode_returns_error() {
        let cache = Arc::new(StaticCodeCache::default());
        let host = RevmInterpreterHost::new(cache);
        let interpreter = make_address(0x44);
        let mut db = InMemoryDB::default();

        let err = host
            .ensure_contract(&mut db, interpreter, BytecodeKind::Interpreter)
            .expect_err("missing bytecode should error");
        match err {
            RaindexError::MissingBytecode { address, kind } => {
                assert_eq!(address, interpreter);
                assert_matches::assert_matches!(kind, BytecodeKind::Interpreter);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ensure_code_loaded_populates_interpreter_and_store_accounts() {
        let cache = Arc::new(StaticCodeCache::default());
        let interpreter = make_address(0x55);
        let store = make_address(0x66);
        let interpreter_code = [0x60, 0x00, 0x60, 0x00, 0x02];
        let store_code = [0x60, 0x01, 0x60, 0x00, 0x03];

        cache.upsert_interpreter(interpreter, &interpreter_code);
        cache.upsert_store(store, &store_code);

        let host = RevmInterpreterHost::new(cache);
        host.ensure_code_loaded(interpreter, store)
            .expect("bytecode should be cached");

        let db = host.base_db.read();
        let interpreter_account = db
            .cache
            .accounts
            .get(&to_revm_address(interpreter))
            .and_then(|account| account.info());
        let store_account = db
            .cache
            .accounts
            .get(&to_revm_address(store))
            .and_then(|account| account.info());

        assert!(
            interpreter_account.is_some(),
            "interpreter should be cached"
        );
        assert!(store_account.is_some(), "store should be cached");
    }
}
