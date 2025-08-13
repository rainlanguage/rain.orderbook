CREATE TABLE deposits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL, -- address of the depositor
    token TEXT NOT NULL, -- token contract address
    vault_id TEXT NOT NULL, -- uint256 as text to handle large numbers
    amount TEXT NOT NULL, -- uint256 as text
    timestamp INTEGER,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_deposits_sender ON deposits(sender);
CREATE INDEX idx_deposits_token ON deposits(token);
CREATE INDEX idx_deposits_vault_id ON deposits(vault_id);
CREATE INDEX idx_deposits_block ON deposits(block_number);

-- Withdraws table: Stores all withdraw events
-- Each row represents a single Withdraw event
CREATE TABLE withdraws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL, -- address of the withdrawer
    token TEXT NOT NULL, -- token contract address
    vault_id TEXT NOT NULL, -- uint256 as text
    target_amount TEXT NOT NULL, -- uint256 as text (requested amount)
    amount TEXT NOT NULL, -- uint256 as text (actual amount withdrawn)
    timestamp INTEGER,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_withdraws_sender ON withdraws(sender);
CREATE INDEX idx_withdraws_token ON withdraws(token);
CREATE INDEX idx_withdraws_vault_id ON withdraws(vault_id);
CREATE INDEX idx_withdraws_block ON withdraws(block_number);

-- ====================================
-- Order-Related Tables
-- ====================================

-- Orders table: Stores AddOrderV2 and RemoveOrderV2 events
-- Links to order_details for the actual OrderV3 struct data
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    event_type TEXT NOT NULL CHECK(event_type IN ('ADD', 'REMOVE')),
    sender TEXT NOT NULL, -- address that added/removed the order
    order_hash TEXT NOT NULL, -- bytes32 order identifier
    timestamp INTEGER,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_orders_order_hash ON orders(order_hash);
CREATE INDEX idx_orders_sender ON orders(sender);
CREATE INDEX idx_orders_event_type ON orders(event_type);
CREATE INDEX idx_orders_block ON orders(block_number);

-- Order_details table: Stores the OrderV3 struct details
-- One row per unique order_hash (orders can be reused across events)
CREATE TABLE order_details (
    order_hash TEXT PRIMARY KEY, -- bytes32 as primary key
    owner TEXT NOT NULL, -- order owner address
    nonce TEXT NOT NULL, -- bytes32 nonce
    evaluable_id INTEGER NOT NULL,
    created_at INTEGER,
    FOREIGN KEY (evaluable_id) REFERENCES evaluables(id)
);

CREATE INDEX idx_order_details_owner ON order_details(owner);

-- Evaluables table: Stores EvaluableV3 struct data
-- Can be reused across multiple orders
CREATE TABLE evaluables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    interpreter TEXT NOT NULL, -- IInterpreterV3 address
    store TEXT NOT NULL, -- IInterpreterStoreV2 address
    bytecode TEXT NOT NULL, -- hex encoded bytecode
    created_at INTEGER
);

CREATE INDEX idx_evaluables_interpreter ON evaluables(interpreter);
CREATE INDEX idx_evaluables_store ON evaluables(store);

-- IOs table: Stores IO struct data
-- Represents valid inputs/outputs for orders
CREATE TABLE ios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT NOT NULL, -- token contract address
    decimals INTEGER NOT NULL CHECK(decimals >= 0 AND decimals <= 255), -- uint8
    vault_id TEXT NOT NULL -- uint256 as text
);

CREATE INDEX idx_ios_token ON ios(token);
CREATE INDEX idx_ios_vault_id ON ios(vault_id);
-- Compound index for deduplication
CREATE UNIQUE INDEX idx_ios_unique ON ios(token, decimals, vault_id);

-- Order_ios junction table: Links orders to their valid inputs/outputs
-- Many-to-many relationship between order_details and ios
CREATE TABLE order_ios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_hash TEXT NOT NULL,
    io_id INTEGER NOT NULL,
    io_type TEXT NOT NULL CHECK(io_type IN ('INPUT', 'OUTPUT')),
    io_index INTEGER NOT NULL, -- position in the array (0-based)
    FOREIGN KEY (order_hash) REFERENCES order_details(order_hash),
    FOREIGN KEY (io_id) REFERENCES ios(id),
    UNIQUE(order_hash, io_type, io_index)
);

CREATE INDEX idx_order_ios_order ON order_ios(order_hash);
CREATE INDEX idx_order_ios_io ON order_ios(io_id);
CREATE INDEX idx_order_ios_type ON order_ios(io_type);

-- ====================================
-- TakeOrder Tables
-- ====================================

-- Take_orders table: Stores TakeOrderV2 events
CREATE TABLE take_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL, -- address that took the order
    order_hash TEXT NOT NULL, -- references the order being taken
    input_io_index INTEGER NOT NULL, -- which input IO was used
    output_io_index INTEGER NOT NULL, -- which output IO was used
    input_amount TEXT NOT NULL, -- uint256 as text
    output_amount TEXT NOT NULL, -- uint256 as text
    timestamp INTEGER,
    FOREIGN KEY (order_hash) REFERENCES order_details(order_hash),
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_take_orders_sender ON take_orders(sender);
CREATE INDEX idx_take_orders_order ON take_orders(order_hash);
CREATE INDEX idx_take_orders_block ON take_orders(block_number);

-- Signed_contexts table: Stores SignedContextV1 data for take orders
CREATE TABLE signed_contexts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    take_order_id INTEGER NOT NULL,
    signer TEXT NOT NULL, -- address of context signer
    signature TEXT NOT NULL, -- hex encoded signature
    context_index INTEGER NOT NULL, -- position in array
    FOREIGN KEY (take_order_id) REFERENCES take_orders(id) ON DELETE CASCADE,
    UNIQUE(take_order_id, context_index)
);

CREATE INDEX idx_signed_contexts_take ON signed_contexts(take_order_id);
CREATE INDEX idx_signed_contexts_signer ON signed_contexts(signer);

-- Signed_context_values table: Stores the uint256[] context array
CREATE TABLE signed_context_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    signed_context_id INTEGER NOT NULL,
    value TEXT NOT NULL, -- uint256 as text
    value_index INTEGER NOT NULL, -- position in context array
    FOREIGN KEY (signed_context_id) REFERENCES signed_contexts(id) ON DELETE CASCADE,
    UNIQUE(signed_context_id, value_index)
);

CREATE INDEX idx_context_values_context ON signed_context_values(signed_context_id);

-- ====================================
-- Clear Tables
-- ====================================

-- Clears table: Stores ClearV2 events
CREATE TABLE clears (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    sender TEXT NOT NULL, -- address that cleared orders
    alice_order_hash TEXT NOT NULL, -- first order
    bob_order_hash TEXT NOT NULL, -- second order
    clear_config_id INTEGER NOT NULL,
    timestamp INTEGER,
    FOREIGN KEY (alice_order_hash) REFERENCES order_details(order_hash),
    FOREIGN KEY (bob_order_hash) REFERENCES order_details(order_hash),
    FOREIGN KEY (clear_config_id) REFERENCES clear_configs(id),
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_clears_sender ON clears(sender);
CREATE INDEX idx_clears_alice ON clears(alice_order_hash);
CREATE INDEX idx_clears_bob ON clears(bob_order_hash);
CREATE INDEX idx_clears_block ON clears(block_number);

-- Clear_configs table: Stores ClearConfig struct data
CREATE TABLE clear_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alice_input_io_index INTEGER NOT NULL,
    alice_output_io_index INTEGER NOT NULL,
    bob_input_io_index INTEGER NOT NULL,
    bob_output_io_index INTEGER NOT NULL,
    alice_bounty_vault_id TEXT NOT NULL, -- uint256 as text
    bob_bounty_vault_id TEXT NOT NULL -- uint256 as text
);

-- ====================================
-- Documentation Views (Optional but helpful)
-- ====================================

-- View to get complete order information with evaluable details
CREATE VIEW v_order_complete AS
SELECT 
    od.order_hash,
    od.owner,
    od.nonce,
    e.interpreter,
    e.store,
    e.bytecode,
    od.created_at
FROM order_details od
JOIN evaluables e ON od.evaluable_id = e.id;

-- View to get order inputs
CREATE VIEW v_order_inputs AS
SELECT 
    oi.order_hash,
    oi.io_index,
    io.token,
    io.decimals,
    io.vault_id
FROM order_ios oi
JOIN ios io ON oi.io_id = io.id
WHERE oi.io_type = 'INPUT'
ORDER BY oi.order_hash, oi.io_index;

-- View to get order outputs
CREATE VIEW v_order_outputs AS
SELECT 
    oi.order_hash,
    oi.io_index,
    io.token,
    io.decimals,
    io.vault_id
FROM order_ios oi
JOIN ios io ON oi.io_id = io.id
WHERE oi.io_type = 'OUTPUT'
ORDER BY oi.order_hash, oi.io_index;