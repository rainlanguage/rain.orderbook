-- Sync status table (for tracking indexer progress)
CREATE TABLE IF NOT EXISTS sync_status (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Ensures only one row
    last_synced_block INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial row
INSERT OR IGNORE INTO sync_status (id, last_synced_block) VALUES (1, 0);

-- Deposit events (IMMUTABLE)
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

CREATE INDEX idx_deposits_sender ON deposits(sender);
CREATE INDEX idx_deposits_token ON deposits(token);
CREATE INDEX idx_deposits_vault ON deposits(vault_id);
CREATE INDEX idx_deposits_block ON deposits(block_number);
CREATE INDEX idx_deposits_tx ON deposits(transaction_hash);

-- Withdraw events (IMMUTABLE)
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

CREATE INDEX idx_withdrawals_sender ON withdrawals(sender);
CREATE INDEX idx_withdrawals_token ON withdrawals(token);
CREATE INDEX idx_withdrawals_vault ON withdrawals(vault_id);
CREATE INDEX idx_withdrawals_block ON withdrawals(block_number);
CREATE INDEX idx_withdrawals_tx ON withdrawals(transaction_hash);

-- Order lifecycle events (IMMUTABLE - stores all state changes)
CREATE TABLE IF NOT EXISTS order_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    event_type TEXT NOT NULL, -- 'add', 'remove', 'take', 'clear'
    sender TEXT NOT NULL,
    order_hash TEXT NOT NULL,
    -- Order data (only populated for 'add' events)
    owner TEXT,
    nonce TEXT,
    interpreter TEXT,
    store TEXT,
    bytecode BLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_order_events_hash ON order_events(order_hash);
CREATE INDEX idx_order_events_type ON order_events(event_type);
CREATE INDEX idx_order_events_sender ON order_events(sender);
CREATE INDEX idx_order_events_block ON order_events(block_number);
CREATE INDEX idx_order_events_tx ON order_events(transaction_hash);

-- IO data for orders (IMMUTABLE)
CREATE TABLE IF NOT EXISTS order_ios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_event_id INTEGER NOT NULL, -- Links to the 'add' event
    io_type TEXT NOT NULL, -- 'input' or 'output'
    io_index INTEGER NOT NULL,
    token TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    vault_id TEXT NOT NULL,
    FOREIGN KEY (order_event_id) REFERENCES order_events(id) ON DELETE CASCADE
);

CREATE INDEX idx_order_ios_event ON order_ios(order_event_id);
CREATE INDEX idx_order_ios_token ON order_ios(token);

-- Take Order events (IMMUTABLE)
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

CREATE INDEX idx_take_orders_sender ON take_orders(sender);
CREATE INDEX idx_take_orders_owner ON take_orders(order_owner);
CREATE INDEX idx_take_orders_nonce ON take_orders(order_nonce);
CREATE INDEX idx_take_orders_block ON take_orders(block_number);
CREATE INDEX idx_take_orders_tx ON take_orders(transaction_hash);

-- Signed contexts for take orders (IMMUTABLE)
CREATE TABLE IF NOT EXISTS take_order_contexts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    take_order_id INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    signer TEXT NOT NULL,
    signature BLOB NOT NULL,
    FOREIGN KEY (take_order_id) REFERENCES take_orders(id) ON DELETE CASCADE,
    UNIQUE(take_order_id, context_index)
);

CREATE INDEX idx_take_contexts_take_id ON take_order_contexts(take_order_id);

-- Context values (IMMUTABLE)
CREATE TABLE IF NOT EXISTS context_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    context_id INTEGER NOT NULL,
    value_index INTEGER NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (context_id) REFERENCES take_order_contexts(id) ON DELETE CASCADE,
    UNIQUE(context_id, value_index)
);

-- Clear events (IMMUTABLE)
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

CREATE INDEX idx_clears_sender ON clears(sender);
CREATE INDEX idx_clears_alice_hash ON clears(alice_order_hash);
CREATE INDEX idx_clears_bob_hash ON clears(bob_order_hash);
CREATE INDEX idx_clears_block ON clears(block_number);
CREATE INDEX idx_clears_tx ON clears(transaction_hash);

-- MetaV1_2 events (IMMUTABLE) - for arbitrary metadata
CREATE TABLE IF NOT EXISTS meta_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT NOT NULL, -- bytes32 stored as hex string
    meta BLOB NOT NULL, -- raw bytes data
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_meta_sender ON meta_events(sender);
CREATE INDEX idx_meta_subject ON meta_events(subject);
CREATE INDEX idx_meta_block ON meta_events(block_number);
CREATE INDEX idx_meta_tx ON meta_events(transaction_hash);
