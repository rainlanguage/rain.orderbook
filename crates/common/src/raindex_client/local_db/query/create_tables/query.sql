BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS sync_status (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_synced_block INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO sync_status (id, last_synced_block) VALUES (1, 0);

CREATE TABLE deposits (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    deposit_amount_uint256 TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE withdrawals (
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
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE order_events (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    order_hash TEXT NOT NULL,
    event_type TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE order_ios (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    io_index INTEGER NOT NULL,
    io_type TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index, io_index, io_type),
    FOREIGN KEY (transaction_hash, log_index) REFERENCES order_events(transaction_hash, log_index)
);

CREATE TABLE take_orders (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    input_io_index INTEGER NOT NULL,
    output_io_index INTEGER NOT NULL,
    input TEXT NOT NULL,
    output TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE take_order_contexts (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    context_value TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index, context_index),
    FOREIGN KEY (transaction_hash, log_index) REFERENCES take_orders(transaction_hash, log_index)
);

CREATE TABLE context_values (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    value_index INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index, context_index, value_index),
    FOREIGN KEY (transaction_hash, log_index, context_index) REFERENCES take_order_contexts(transaction_hash, log_index, context_index)
);

CREATE TABLE clear_v3_events (
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
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE after_clear_v2_events (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_output TEXT NOT NULL,
    bob_output TEXT NOT NULL,
    alice_input TEXT NOT NULL,
    bob_input TEXT NOT NULL,
    PRIMARY KEY (transaction_hash, log_index)
);

CREATE TABLE meta_events (
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT NOT NULL,
    meta BLOB NOT NULL,
    PRIMARY KEY (transaction_hash, log_index)
);
CREATE INDEX idx_deposits_vault ON deposits(sender, token, vault_id);
CREATE INDEX idx_deposits_block ON deposits(block_number);
CREATE INDEX idx_deposits_token ON deposits(token);

CREATE INDEX idx_withdrawals_vault ON withdrawals(sender, token, vault_id);
CREATE INDEX idx_withdrawals_block ON withdrawals(block_number);
CREATE INDEX idx_withdrawals_token ON withdrawals(token);

CREATE INDEX idx_order_events_hash ON order_events(order_hash);
CREATE INDEX idx_order_events_owner ON order_events(order_owner);
CREATE INDEX idx_order_events_block ON order_events(block_number);

CREATE INDEX idx_order_ios_token ON order_ios(token);

CREATE INDEX idx_take_orders_owner ON take_orders(order_owner);
CREATE INDEX idx_take_orders_block ON take_orders(block_number);

CREATE INDEX idx_clear_events_alice_bob ON clear_v3_events(alice_order_hash, bob_order_hash);
CREATE INDEX idx_clear_events_block ON clear_v3_events(block_number);
CREATE INDEX idx_clear_alice_vaults ON clear_v3_events(alice_input_vault_id, alice_output_vault_id);
CREATE INDEX idx_clear_bob_vaults ON clear_v3_events(bob_input_vault_id, bob_output_vault_id);

CREATE INDEX idx_after_clear_block ON after_clear_v2_events(block_number);

CREATE INDEX idx_meta_subject ON meta_events(subject);
CREATE INDEX idx_meta_block ON meta_events(block_number);

COMMIT;
