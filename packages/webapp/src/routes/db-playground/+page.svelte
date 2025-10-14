<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { PageHeader, useRaindexClient } from '@rainlanguage/ui-components';
	import { Button, Textarea } from 'flowbite-svelte';
	import init, { SQLiteWasmDatabase, type WasmEncodedResult } from '@rainlanguage/sqlite-web';
	import { type LocalDb } from '@rainlanguage/orderbook';

	let raindexClient = useRaindexClient();
	let localDbClient = raindexClient.getLocalDbClient(42161).value as LocalDb;

	let db: WasmEncodedResult<SQLiteWasmDatabase> | null = null;
	let sqlQuery = '';
	let queryResults: unknown = null;
	let isLoading = false;
	let error = '';

	// Sync status message from raindexClient.syncLocalDatabaseOnce callback
	let syncStatus: string = '';

	// Auto-sync state variables
	let autoSyncEnabled = false;
	let lastSyncedBlock: string | null = null;
	let lastSyncTime: Date | null = null;
	let autoSyncInterval: ReturnType<typeof setInterval> | null = null;

	// Whether a sync operation is actively running
	let isSyncing = false;

	let showCustomQuery = false;

	onMount(async () => {
		await init();
		db = SQLiteWasmDatabase.new('rainlanguage_orderbook_webapp');

		if (db && !db.error && db.value) {
			const queryFn = db.value.query.bind(db.value);
			raindexClient.setDbCallback(queryFn);
		}

		// Populate last sync info on load
		await updateSyncStatus();

		// Auto-start syncing after db is initialized
		if (db && !db.error && db.value) {
			// await startAutoSync();
		}
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

	async function startAutoSync() {
		if (!db?.value || autoSyncEnabled) return;

		autoSyncEnabled = true;

		// Initial sync
		await performAutoSync();
		// Get current sync status
		await updateSyncStatus();

		// Set up interval for every 5 seconds
		autoSyncInterval = setInterval(async () => {
			await performAutoSync();
		}, 5000);
	}

	async function updateSyncStatus() {
		if (!db?.value) return;

		try {
			error = '';
			const queryFn = db.value.query.bind(db.value);
			const statusResult = await localDbClient.getSyncStatus(queryFn);

			if (!statusResult.error && statusResult.value) {
				const statusArray = statusResult.value;
				if (statusArray && statusArray.length > 0) {
					const latestStatus = statusArray[statusArray.length - 1];
					lastSyncedBlock = latestStatus.last_synced_block.toString();
					lastSyncTime = latestStatus.updated_at ? new Date(latestStatus.updated_at) : new Date();
				}
			}
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		}
	}

	function stopAutoSync() {
		autoSyncEnabled = false;
		if (autoSyncInterval) {
			clearInterval(autoSyncInterval);
			autoSyncInterval = null;
		}
	}

	async function performAutoSync() {
		if (!db?.value || isLoading) return;

		try {
			isSyncing = true;
			const queryFn = db.value.query.bind(db.value);

			// Sync database and capture status updates
			const syncResult = await raindexClient.syncLocalDatabaseOnce(
				queryFn,
				(status: string) => {
					// Update the UI with latest status message
					syncStatus = status;
				},
				42161
			);

			if (syncResult.error) {
				error = syncResult.error.msg;
				return;
			}

			error = '';
			// Update sync status display
			await updateSyncStatus();
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			isSyncing = false;
		}
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
		stopAutoSync();
	});
</script>

<PageHeader title="Database Playground" pathname={$page.url.pathname} />

{#if db}
	{#if db.error}
		<div class="mx-auto max-w-6xl p-4">
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
		<div class="mx-auto max-w-6xl space-y-6 p-4">
			<div
				class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
			>
				<div class="mb-4">
					<div class="mb-4 flex items-center justify-between">
						<h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">
							Database Operations
						</h3>
						<div class="flex items-center gap-3">
							{#if autoSyncEnabled || lastSyncedBlock}
								<div class="text-right text-sm">
									<div class="flex items-center text-blue-700 dark:text-blue-300">
										{#if autoSyncEnabled}
											<div class="mr-2 h-2 w-2 animate-pulse rounded-full bg-green-500"></div>
											<span class="font-medium">Auto-Sync Active</span>
										{:else}
											<div class="mr-2 h-2 w-2 rounded-full bg-gray-400"></div>
											<span class="font-medium text-gray-500">Auto-Sync Stopped</span>
										{/if}
									</div>
									{#if isSyncing && syncStatus}
										<div class="mt-1 text-xs text-gray-600 dark:text-gray-400">
											Status: {syncStatus}
										</div>
									{:else if lastSyncedBlock}
										<div class="mt-1 text-xs text-gray-600 dark:text-gray-400">
											Last sync: block {lastSyncedBlock}
											{#if lastSyncTime}
												at {lastSyncTime.toLocaleString()}
											{/if}
										</div>
									{/if}
								</div>
							{/if}
							<button
								on:click={() => {
									if (autoSyncEnabled) {
										stopAutoSync();
									} else {
										startAutoSync();
									}
								}}
								disabled={isLoading}
								class="flex h-8 w-8 items-center justify-center rounded-full border-2 transition-colors {autoSyncEnabled
									? 'border-red-500 bg-red-500 text-white hover:bg-red-600'
									: 'border-green-500 bg-green-500 text-white hover:bg-green-600'} {isLoading
									? 'cursor-not-allowed opacity-50'
									: 'cursor-pointer'}"
								title={autoSyncEnabled ? 'Stop Auto-Sync' : 'Start Auto-Sync'}
							>
								{#if autoSyncEnabled}
									<!-- Stop/Pause Icon -->
									<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
										<rect x="6" y="4" width="4" height="16" />
										<rect x="14" y="4" width="4" height="16" />
									</svg>
								{:else}
									<!-- Play Icon -->
									<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
										<polygon points="5,3 19,12 5,21" />
									</svg>
								{/if}
							</button>
						</div>
					</div>
				</div>

				<div class="mb-6">
					<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
						Database Management
					</h4>
					<div class="flex flex-wrap gap-2">
						<Button
							on:click={() => executeRaindexQuery(() => localDbClient.clearTables(queryFn))}
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
					<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
						Order Management
					</h4>
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
							class="ml-2 h-4 w-4 transform transition-transform {showCustomQuery
								? 'rotate-180'
								: ''}"
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
	{/if}
{:else}
	<div class="mx-auto max-w-6xl p-4">
		<div
			class="rounded-lg border border-gray-200 bg-white p-6 text-center shadow-sm dark:border-gray-700 dark:bg-gray-800"
		>
			<p class="text-gray-600 dark:text-gray-300">Loading SQLite Worker...</p>
		</div>
	</div>
{/if}
