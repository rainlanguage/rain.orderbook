<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { PageHeader } from '@rainlanguage/ui-components';
	import { Button, Textarea } from 'flowbite-svelte';
	import init, { SQLiteWasmDatabase, type WasmEncodedResult } from 'sqlite-worker';
	import { RaindexClient } from '@rainlanguage/orderbook';

	let db: WasmEncodedResult<SQLiteWasmDatabase> | null = null;
	let sqlQuery = '';
	let queryResults: unknown = null;
	let isLoading = false;
	let error = '';

	onMount(async () => {
		await init();
		db = SQLiteWasmDatabase.new();
	});

	async function executeQuery() {
		if (!db?.value || !sqlQuery.trim()) return;

		isLoading = true;
		error = '';
		queryResults = null;

		// Split SQL by semicolons and filter out empty statements
		const statements = sqlQuery
			.split(';')
			.map((stmt) => stmt.trim())
			.filter((stmt) => stmt.length > 0);

		const allResults = [];

		for (const statement of statements) {
			const result = await db.value.query(statement);
			if (result.error) {
				error = `Error in statement "${statement}": ${result.error.msg}`;
				break;
			} else {
				try {
					const parsedResult = JSON.parse(result.value);
					allResults.push({
						statement,
						result: parsedResult
					});
				} catch {
					allResults.push({
						statement,
						result: result.value
					});
				}
			}
		}

		if (!error) {
			queryResults = statements.length === 1 ? allResults[0]?.result : allResults;
		}

		isLoading = false;
	}

	function executeCommonQuery(query: string) {
		sqlQuery = query;
		executeQuery();
	}

	async function syncDatabase() {
		try {
			isLoading = true;
			await RaindexClient.syncDatabase(async (data: string) => {
				executeCommonQuery(data);
			});
		} catch (error) {
			console.error('Error syncing database:', error);
		}
	}
</script>

<PageHeader title="Database Playground" pathname={$page.url.pathname} />

{#if db}
	{#if db.error}
		<div class="mx-auto max-w-4xl p-4">
			<div
				class="rounded-lg border border-red-300 bg-red-50 p-4 dark:border-red-600 dark:bg-red-900/20"
			>
				<h3 class="text-lg font-medium text-red-800 dark:text-red-200">
					Error initializing SQLite Worker:
				</h3>
				<pre class="mt-2 text-sm text-red-700 dark:text-red-300">{db.error.msg}</pre>
			</div>
		</div>
	{:else if db.value}
		<div class="mx-auto max-w-4xl space-y-6 p-4">
			<div
				class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
			>
				<div class="mb-4">
					<h3 class="mb-3 text-lg font-medium text-gray-900 dark:text-gray-100">Common Queries</h3>
					<div class="mb-6 flex flex-wrap gap-2">
						<Button
							on:click={() =>
								executeCommonQuery(`-- Sync status table (for tracking indexer progress)
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
CREATE INDEX idx_meta_tx ON meta_events(transaction_hash);`)}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Create Tables
						</Button>
						<Button on:click={syncDatabase} disabled={isLoading} color="light" size="sm">
							Sync Database
						</Button>
						<Button
							on:click={() =>
								executeCommonQuery("SELECT name FROM sqlite_master WHERE type='table';")}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Fetch All Tables
						</Button>
						<!-- <Button
							on:click={() =>
								executeCommonQuery("SELECT sql FROM sqlite_master WHERE type='table';")}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Show Table Schemas
						</Button> -->
						<!-- <Button
							on:click={() =>
								executeCommonQuery("SELECT name FROM sqlite_master WHERE type='view';")}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Fetch All Views
						</Button> -->
						<Button
							on:click={() =>
								executeCommonQuery(`SELECT
      add_events.order_hash,
      add_events.owner,
      add_events.block_timestamp as creation_time,
      GROUP_CONCAT(
          CASE WHEN ios.io_type = 'input' THEN
              ios.token || ':' || ios.decimals || ':' || ios.vault_id
          END
      ) as inputs,
      GROUP_CONCAT(
          CASE WHEN ios.io_type = 'output' THEN
              ios.token || ':' || ios.decimals || ':' || ios.vault_id
          END
      ) as outputs,
      (
          SELECT COUNT(*)
          FROM take_orders
          WHERE order_owner = add_events.owner
            AND order_nonce = add_events.nonce
      ) as trade_count
  FROM order_events as add_events
  LEFT JOIN order_ios as ios ON add_events.id = ios.order_event_id
  WHERE add_events.event_type = 'add'
    AND add_events.order_hash NOT IN (
        SELECT order_hash
        FROM order_events
        WHERE event_type = 'remove'
    )
  GROUP BY add_events.order_hash, add_events.owner, add_events.block_timestamp,
  add_events.nonce
  ORDER BY add_events.block_timestamp DESC;`)}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Get Active Orders
						</Button>
						<Button
							on:click={() =>
								executeCommonQuery(`SELECT
    d.vault_id,
    d.sender as owner,
    d.token,
    'TODO: balance calculation' as balance,
    GROUP_CONCAT(
        CASE WHEN ios.io_type = 'input' THEN
            ios_order.order_hash
        END
    ) as input_for_orders,
    GROUP_CONCAT(
        CASE WHEN ios.io_type = 'output' THEN
            ios_order.order_hash
        END
    ) as output_for_orders
FROM deposits d
LEFT JOIN order_ios ios ON d.vault_id = ios.vault_id AND d.token = ios.token
LEFT JOIN order_events ios_order ON ios.order_event_id = ios_order.id
WHERE ios_order.event_type = 'add' OR ios_order.event_type IS NULL
GROUP BY d.vault_id, d.sender, d.token
ORDER BY d.vault_id;`)}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Get All Vaults
						</Button>
						<Button
							on:click={() =>
								executeCommonQuery(`
  -- Drop tables with foreign keys first
  DROP TABLE IF EXISTS context_values;
  DROP TABLE IF EXISTS take_order_contexts;
  DROP TABLE IF EXISTS take_orders;
  DROP TABLE IF EXISTS order_ios;
  DROP TABLE IF EXISTS order_events;
  DROP TABLE IF EXISTS clears;
  DROP TABLE IF EXISTS meta_events;
  DROP TABLE IF EXISTS withdrawals;
  DROP TABLE IF EXISTS deposits;

  -- Drop the core metadata table
  DROP TABLE IF EXISTS events_metadata;

  -- Drop the sync status table
  DROP TABLE IF EXISTS sync_status;

  -- Vacuum
  VACUUM;
								`)}
							disabled={isLoading}
							color="red"
							size="sm"
						>
							Clear All Tables & Views
						</Button>
					</div>
				</div>

				<div class="mb-4">
					<label
						for="sql-query"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						SQL Query
					</label>
					<Textarea
						id="sql-query"
						bind:value={sqlQuery}
						placeholder="Enter your SQL query here..."
						rows="6"
						class="font-mono"
					/>
				</div>

				<Button
					on:click={executeQuery}
					disabled={isLoading || !sqlQuery.trim()}
					color="blue"
					size="lg"
				>
					{isLoading ? 'Executing...' : 'Execute Query'}
				</Button>
			</div>

			{#if error}
				<div
					class="rounded-lg border border-red-300 bg-white p-6 shadow-sm dark:border-red-600 dark:bg-gray-800"
				>
					<h3 class="mb-3 text-lg font-medium text-red-800 dark:text-red-200">Error</h3>
					<pre
						class="overflow-x-auto rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-300">{error}</pre>
				</div>
			{/if}

			{#if queryResults}
				<div
					class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
				>
					<h3 class="mb-3 text-lg font-medium text-gray-900 dark:text-gray-100">Query Results</h3>
					<pre
						class="max-h-96 overflow-x-auto overflow-y-auto whitespace-pre-wrap rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm leading-relaxed dark:border-gray-600 dark:bg-gray-900">{JSON.stringify(
							queryResults,
							null,
							2
						)}</pre>
				</div>
			{/if}
		</div>
	{/if}
{:else}
	<div class="mx-auto max-w-4xl p-4">
		<div
			class="rounded-lg border border-gray-200 bg-white p-6 text-center shadow-sm dark:border-gray-700 dark:bg-gray-800"
		>
			<p class="text-gray-600 dark:text-gray-300">Loading SQLite Worker...</p>
		</div>
	</div>
{/if}
