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

	async function executeRaindexQuery(queryFunction: () => Promise<WasmEncodedResult<unknown>>) {
		if (!db?.value) {
			error = 'Database not initialized';
			return;
		}

		isLoading = true;
		error = '';
		queryResults = null;

		const result = await queryFunction();
		if (result.error) {
			error = `Error executing query: ${result.error.msg}`;
		} else {
			queryResults = result.value;
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
		{@const queryFn = db.value.query.bind(db.value)}
		<div class="mx-auto max-w-4xl space-y-6 p-4">
			<div
				class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
			>
				<div class="mb-4">
					<h3 class="mb-3 text-lg font-medium text-gray-900 dark:text-gray-100">Common Queries</h3>
					<div class="mb-6 flex flex-wrap gap-2">
						<Button
							on:click={() => executeRaindexQuery(() => RaindexClient.createTables(queryFn))}
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
							on:click={() => {
								executeRaindexQuery(() => RaindexClient.fetchAllTables(queryFn));
							}}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Fetch All Tables
						</Button>
						<Button
							on:click={() => executeRaindexQuery(() => RaindexClient.getActiveOrders(queryFn))}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Get Active Orders
						</Button>
						<Button
							on:click={() => {
								executeRaindexQuery(() =>
									RaindexClient.getOrderTrades(queryFn, prompt('Enter order hash:') || '')
								);
							}}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Get Trades for Order
						</Button>
						<Button
							on:click={() => {
								executeRaindexQuery(() =>
									RaindexClient.getOrderVaultVolumes(queryFn, prompt('Enter order hash:') || '')
								);
							}}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Get Order Vault Volumes
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
							on:click={() => executeRaindexQuery(() => RaindexClient.clearTables(queryFn))}
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
