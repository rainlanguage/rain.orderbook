CREATE TABLE IF NOT EXISTS sync_status (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_synced_block INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO sync_status (id, last_synced_block) VALUES (1, 0);

CREATE TABLE IF NOT EXISTS deposits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    amount TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_deposits_sender ON deposits(sender);
CREATE INDEX IF NOT EXISTS idx_deposits_token ON deposits(token);
CREATE INDEX IF NOT EXISTS idx_deposits_vault ON deposits(vault_id);
CREATE INDEX IF NOT EXISTS idx_deposits_block ON deposits(block_number);
CREATE INDEX IF NOT EXISTS idx_deposits_tx ON deposits(transaction_hash);

CREATE TABLE IF NOT EXISTS withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    target_amount TEXT NOT NULL,
    amount TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_withdrawals_sender ON withdrawals(sender);
CREATE INDEX IF NOT EXISTS idx_withdrawals_token ON withdrawals(token);
CREATE INDEX IF NOT EXISTS idx_withdrawals_vault ON withdrawals(vault_id);
CREATE INDEX IF NOT EXISTS idx_withdrawals_block ON withdrawals(block_number);
CREATE INDEX IF NOT EXISTS idx_withdrawals_tx ON withdrawals(transaction_hash);

CREATE TABLE IF NOT EXISTS order_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    sender TEXT NOT NULL,
    order_hash TEXT NOT NULL,
    owner TEXT,
    nonce TEXT,
    interpreter TEXT,
    store TEXT,
    bytecode BLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_order_events_hash ON order_events(order_hash);
CREATE INDEX IF NOT EXISTS idx_order_events_type ON order_events(event_type);
CREATE INDEX IF NOT EXISTS idx_order_events_sender ON order_events(sender);
CREATE INDEX IF NOT EXISTS idx_order_events_block ON order_events(block_number);
CREATE INDEX IF NOT EXISTS idx_order_events_tx ON order_events(transaction_hash);

CREATE TABLE IF NOT EXISTS order_ios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_event_id INTEGER NOT NULL,
    io_type TEXT NOT NULL,
    io_index INTEGER NOT NULL,
    token TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    vault_id TEXT NOT NULL,
    FOREIGN KEY (order_event_id) REFERENCES order_events(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_order_ios_event ON order_ios(order_event_id);
CREATE INDEX IF NOT EXISTS idx_order_ios_token ON order_ios(token);

CREATE TABLE IF NOT EXISTS take_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    input_io_index INTEGER NOT NULL,
    output_io_index INTEGER NOT NULL,
    input_amount TEXT NOT NULL,
    output_amount TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_take_orders_sender ON take_orders(sender);
CREATE INDEX IF NOT EXISTS idx_take_orders_owner ON take_orders(order_owner);
CREATE INDEX IF NOT EXISTS idx_take_orders_nonce ON take_orders(order_nonce);
CREATE INDEX IF NOT EXISTS idx_take_orders_block ON take_orders(block_number);
CREATE INDEX IF NOT EXISTS idx_take_orders_tx ON take_orders(transaction_hash);

CREATE TABLE IF NOT EXISTS take_order_contexts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    take_order_id INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    signer TEXT NOT NULL,
    signature BLOB NOT NULL,
    FOREIGN KEY (take_order_id) REFERENCES take_orders(id) ON DELETE CASCADE,
    UNIQUE(take_order_id, context_index)
);

CREATE INDEX IF NOT EXISTS idx_take_contexts_take_id ON take_order_contexts(take_order_id);

CREATE TABLE IF NOT EXISTS context_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    context_id INTEGER NOT NULL,
    value_index INTEGER NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (context_id) REFERENCES take_order_contexts(id) ON DELETE CASCADE,
    UNIQUE(context_id, value_index)
);

CREATE TABLE IF NOT EXISTS clears (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_order_hash TEXT NOT NULL,
    bob_order_hash TEXT NOT NULL,
    alice_input_io_index INTEGER NOT NULL,
    alice_output_io_index INTEGER NOT NULL,
    bob_input_io_index INTEGER NOT NULL,
    bob_output_io_index INTEGER NOT NULL,
    alice_bounty_vault_id TEXT NOT NULL,
    bob_bounty_vault_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_clears_sender ON clears(sender);
CREATE INDEX IF NOT EXISTS idx_clears_alice_hash ON clears(alice_order_hash);
CREATE INDEX IF NOT EXISTS idx_clears_bob_hash ON clears(bob_order_hash);
CREATE INDEX IF NOT EXISTS idx_clears_block ON clears(block_number);
CREATE INDEX IF NOT EXISTS idx_clears_tx ON clears(transaction_hash);

CREATE TABLE IF NOT EXISTS meta_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT NOT NULL,
    meta BLOB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_meta_sender ON meta_events(sender);
CREATE INDEX IF NOT EXISTS idx_meta_subject ON meta_events(subject);
CREATE INDEX IF NOT EXISTS idx_meta_block ON meta_events(block_number);
CREATE INDEX IF NOT EXISTS idx_meta_tx ON meta_events(transaction_hash);