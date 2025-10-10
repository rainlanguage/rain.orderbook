BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS sync_status (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    last_synced_block INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chain_id, orderbook_address)
);

CREATE TABLE IF NOT EXISTS raw_events (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER,
    address TEXT NOT NULL,
    topics TEXT NOT NULL,
    data TEXT NOT NULL,
    raw_json TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_raw_events_block ON raw_events(chain_id, orderbook_address, block_number, log_index);
CREATE INDEX IF NOT EXISTS idx_raw_events_address ON raw_events(chain_id, orderbook_address, address);

CREATE TABLE IF NOT EXISTS deposits (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    deposit_amount TEXT NOT NULL,
    deposit_amount_uint256 TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_deposits_vault ON deposits(chain_id, orderbook_address, sender, token, vault_id);
CREATE INDEX IF NOT EXISTS idx_deposits_block ON deposits(chain_id, orderbook_address, block_number);
CREATE INDEX IF NOT EXISTS idx_deposits_token ON deposits(chain_id, orderbook_address, token);

CREATE TABLE IF NOT EXISTS withdrawals (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    target_amount TEXT NOT NULL,
    withdraw_amount TEXT NOT NULL,
    withdraw_amount_uint256 TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_withdrawals_vault ON withdrawals(chain_id, orderbook_address, sender, token, vault_id);
CREATE INDEX IF NOT EXISTS idx_withdrawals_block ON withdrawals(chain_id, orderbook_address, block_number);
CREATE INDEX IF NOT EXISTS idx_withdrawals_token ON withdrawals(chain_id, orderbook_address, token);

CREATE TABLE IF NOT EXISTS order_events (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    interpreter_address TEXT NOT NULL,
    store_address TEXT NOT NULL,
    order_hash TEXT NOT NULL,
    event_type TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    order_bytes TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_order_events_hash ON order_events(chain_id, orderbook_address, order_hash);
CREATE INDEX IF NOT EXISTS idx_order_events_owner ON order_events(chain_id, orderbook_address, order_owner);
CREATE INDEX IF NOT EXISTS idx_order_events_block ON order_events(chain_id, orderbook_address, block_number);
CREATE INDEX IF NOT EXISTS idx_order_events_store ON order_events(chain_id, orderbook_address, store_address);

CREATE TABLE IF NOT EXISTS order_ios (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    io_index INTEGER NOT NULL,
    io_type TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index, io_index, io_type),
    FOREIGN KEY (chain_id, orderbook_address, transaction_hash, log_index)
        REFERENCES order_events(chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_order_ios_token ON order_ios(chain_id, orderbook_address, token);

CREATE TABLE IF NOT EXISTS take_orders (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    input_io_index INTEGER NOT NULL,
    output_io_index INTEGER NOT NULL,
    taker_input TEXT NOT NULL,
    taker_output TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_take_orders_owner ON take_orders(chain_id, orderbook_address, order_owner);
CREATE INDEX IF NOT EXISTS idx_take_orders_block ON take_orders(chain_id, orderbook_address, block_number);

CREATE TABLE IF NOT EXISTS take_order_contexts (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    context_value TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index, context_index),
    FOREIGN KEY (chain_id, orderbook_address, transaction_hash, log_index)
        REFERENCES take_orders(chain_id, orderbook_address, transaction_hash, log_index)
);

CREATE TABLE IF NOT EXISTS context_values (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    value_index INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (
        chain_id,
        orderbook_address,
        transaction_hash,
        log_index,
        context_index,
        value_index
    ),
    FOREIGN KEY (chain_id, orderbook_address, transaction_hash, log_index, context_index)
        REFERENCES take_order_contexts(
            chain_id,
            orderbook_address,
            transaction_hash,
            log_index,
            context_index
        )
);

CREATE TABLE IF NOT EXISTS clear_v3_events (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_order_hash TEXT NOT NULL,
    alice_order_owner TEXT NOT NULL,
    alice_input_io_index INTEGER NOT NULL,
    alice_output_io_index INTEGER NOT NULL,
    alice_bounty_vault_id TEXT NOT NULL,
    alice_input_vault_id TEXT NOT NULL,
    alice_output_vault_id TEXT NOT NULL,
    bob_order_hash TEXT NOT NULL,
    bob_order_owner TEXT NOT NULL,
    bob_input_io_index INTEGER NOT NULL,
    bob_output_io_index INTEGER NOT NULL,
    bob_bounty_vault_id TEXT NOT NULL,
    bob_input_vault_id TEXT NOT NULL,
    bob_output_vault_id TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);

CREATE TABLE IF NOT EXISTS after_clear_v2_events (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_output TEXT NOT NULL,
    bob_output TEXT NOT NULL,
    alice_input TEXT NOT NULL,
    bob_input TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_after_clear_block ON after_clear_v2_events(chain_id, orderbook_address, block_number);

CREATE TABLE IF NOT EXISTS meta_events (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT NOT NULL,
    meta BLOB NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_meta_subject ON meta_events(chain_id, orderbook_address, subject);
CREATE INDEX IF NOT EXISTS idx_meta_block ON meta_events(chain_id, orderbook_address, block_number);

CREATE TABLE IF NOT EXISTS erc20_tokens (
    chain_id INTEGER NOT NULL,
    address  TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    symbol   TEXT    NOT NULL,
    decimals INTEGER NOT NULL,
    PRIMARY KEY (chain_id, address)
);

CREATE TABLE IF NOT EXISTS interpreter_store_sets (
    chain_id INTEGER NOT NULL,
    orderbook_address TEXT NOT NULL,
    store_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (chain_id, orderbook_address, transaction_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_store_sets_store ON interpreter_store_sets(chain_id, orderbook_address, store_address);
CREATE INDEX IF NOT EXISTS idx_store_sets_block ON interpreter_store_sets(chain_id, orderbook_address, block_number);
CREATE INDEX IF NOT EXISTS idx_store_sets_namespace ON interpreter_store_sets(chain_id, orderbook_address, namespace);

COMMIT;
