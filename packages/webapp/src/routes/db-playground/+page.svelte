<script lang="ts">
	import { onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { PageHeader, useLocalDb, useRaindexClient } from '@rainlanguage/ui-components';
	import { Button, Textarea } from 'flowbite-svelte';
	import { SQLiteWasmDatabase, type WasmEncodedResult } from '@rainlanguage/sqlite-web';
	import { clearTables } from '@rainlanguage/orderbook';

	const localDb = useLocalDb();
	const raindexClient = useRaindexClient();

	let db: WasmEncodedResult<SQLiteWasmDatabase> | null = null;
	let sqlQuery = '';
	let queryResults: unknown = null;
	let isLoading = false;
	let error = '';

	// Show/hide advanced SQL query editor
	let showCustomQuery = false;

	async function executeQuery() {
		isLoading = true;
		error = '';
		queryResults = null;

		const result = await localDb.query(sqlQuery);
		if (result.error) {
			error = result.error.msg;
			isLoading = false;
			return;
		}

		try {
			queryResults = JSON.parse(result.value);
		} catch {
			queryResults = result.value;
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

	// Fetch all orders using raindexClient
	async function fetchAllOrders() {
		isLoading = true;
		error = '';
		queryResults = null;

		try {
			const result = await raindexClient.getOrders([42161], null, 1);
			if (result.error) {
				error = result.error.readableMsg ?? result.error.msg;
				return;
			}

			for (const order of result.value) {
				// eslint-disable-next-line no-console
				console.log('Order active:', order.active);
				// eslint-disable-next-line no-console
				console.log('Order hash:', order.orderHash);
				// eslint-disable-next-line no-console
				console.log('Order bytes:', order.orderBytes);

				for (const input of order.inputsList.items) {
					// eslint-disable-next-line no-console
					console.log('Input vault id: ', input.vaultId);
					// eslint-disable-next-line no-console
					console.log(
						`Token: ${input.token.symbol} (${input.token.address}) - ${input.token.decimals}`
					);
					// eslint-disable-next-line no-console
					console.log('Balance: ', input.balance.format().value);
					// eslint-disable-next-line no-console
					console.log('Orders as input:', input.ordersAsInput);
					// eslint-disable-next-line no-console
					console.log('\n');
				}

				for (const output of order.outputsList.items) {
					// eslint-disable-next-line no-console
					console.log('Output vault id: ', output.vaultId);
					// eslint-disable-next-line no-console
					console.log(
						`Token: ${output.token.symbol} (${output.token.address}) - ${output.token.decimals}`
					);
					// eslint-disable-next-line no-console
					console.log('Balance: ', output.balance.format().value);
					// eslint-disable-next-line no-console
					console.log('\n');
				}

				for (const vault of order.vaultsList.items) {
					// eslint-disable-next-line no-console
					console.log('Input/Output vault id: ', vault.vaultId);
					// eslint-disable-next-line no-console
					console.log(
						`Token: ${vault.token.symbol} (${vault.token.address}) - ${vault.token.decimals}`
					);
					// eslint-disable-next-line no-console
					console.log('Balance: ', vault.balance.format().value);
					// eslint-disable-next-line no-console
					console.log('\n');
				}

				// eslint-disable-next-line no-console
				console.log('Order trade count:', order.tradesCount);
				// eslint-disable-next-line no-console
				console.log('\n\n');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			isLoading = false;
		}
	}

	onDestroy(() => {
		// Nothing to clean up currently; placeholder for future resources.
	});
</script>

<PageHeader title="Database Playground" pathname={$page.url.pathname} />

<div class="mx-auto max-w-6xl space-y-6 p-4">
	<div
		class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
	>
		<h3 class="mb-4 text-lg font-medium text-gray-900 dark:text-gray-100">Database Operations</h3>

		<div class="mb-6">
			<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">Database Management</h4>
			<div class="flex flex-wrap gap-2">
				<Button
					on:click={() => executeRaindexQuery(() => clearTables(queryFn))}
					disabled={isLoading}
					color="red"
					size="sm"
				>
					Clear All Tables
				</Button>
			</div>
		</div>

		<!-- Order Management Section -->
		<div class="mb-6">
			<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">Order Management</h4>
			<div class="flex flex-wrap gap-2">
				<Button on:click={fetchAllOrders} disabled={isLoading} color="blue" size="sm">
					Fetch All Orders
				</Button>
			</div>
		</div>

		<!-- Custom SQL Query Section -->
		<div class="mb-4">
			<Button
				on:click={() => (showCustomQuery = !showCustomQuery)}
				color="light"
				size="sm"
				class="mb-3"
			>
				{showCustomQuery ? 'Hide' : 'Show'} Custom SQL Query
				<svg
					class="ml-2 h-4 w-4 transform transition-transform {showCustomQuery ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 9l-7 7-7-7"
					/>
				</svg>
			</Button>

			{#if showCustomQuery}
				<div
					class="space-y-4 rounded-lg border border-gray-200 bg-gray-50 p-4 dark:border-gray-600 dark:bg-gray-800"
				>
					<div>
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
			{/if}
		</div>
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
