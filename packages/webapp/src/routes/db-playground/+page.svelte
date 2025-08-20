<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { PageHeader } from '@rainlanguage/ui-components';
	import { Button, Textarea, Input, Label } from 'flowbite-svelte';
	import init, { SQLiteWasmDatabase, type WasmEncodedResult } from 'sqlite-worker';
	import { RaindexClient } from '@rainlanguage/orderbook';

	let db: WasmEncodedResult<SQLiteWasmDatabase> | null = null;
	let sqlQuery = '';
	let queryResults: unknown = null;
	let isLoading = false;
	let error = '';
	let syncStatus = '';

	// Auto-sync state variables
	let autoSyncEnabled = false;
	let lastSyncedBlock: string | null = null;
	let lastSyncTime: Date | null = null;
	let autoSyncInterval: ReturnType<typeof setInterval> | null = null;

	// Input fields for parameterized queries
	let showOrderHashInput = false;
	let showVaultHistoryInputs = false;
	let orderHashInput = '';
	let vaultIdInput = '';
	let tokenAddressInput = '';
	let orderHashInputType = ''; // 'trades' or 'volumes'
	let showCustomQuery = false;

	onMount(async () => {
		await init();
		db = SQLiteWasmDatabase.new();

		// Auto-start syncing after db is initialized
		if (db && !db.error && db.value) {
			await startAutoSync();
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
		syncStatus = ''; // Clear sync status when operation completes
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
			const queryFn = db.value.query.bind(db.value);
			const statusResult = await RaindexClient.getSyncStatus(queryFn);

			console.log('Sync status result:', statusResult); // Debug log

			if (!statusResult.error && statusResult.value) {
				const statusArray = statusResult.value;
				console.log('Status array:', statusArray); // Debug log

				if (statusArray && statusArray.length > 0) {
					const latestStatus = statusArray[statusArray.length - 1];
					lastSyncedBlock = latestStatus.lastSyncedBlock.toString();
					lastSyncTime = latestStatus.updatedAt ? new Date(latestStatus.updatedAt) : new Date();
					console.log('Updated status:', { lastSyncedBlock, lastSyncTime }); // Debug log
				}
			} else {
				console.log('Error getting sync status:', statusResult.error);
			}
		} catch (err) {
			console.error('Failed to get sync status:', err);
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
			const queryFn = db.value.query.bind(db.value);

			// Sync database
			const syncResult = await RaindexClient.syncDatabase(
				queryFn,
				() => {}, // Don't update syncStatus for auto-sync
				'0xd2938e7c9fe3597f78832ce780feb61945c377d7', // contract address
				BigInt(19033330) // default start block
			);

			if (syncResult.error) {
				console.error('Auto-sync error:', syncResult.error.msg);
				return;
			}

			// Update sync status display
			await updateSyncStatus();
		} catch (err) {
			console.error('Auto-sync failed:', err);
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
									<div class="text-xs text-blue-600 dark:text-blue-400">
										Block: {lastSyncedBlock || 'Loading...'} | Last: {lastSyncTime
											? lastSyncTime.toLocaleTimeString()
											: 'Loading...'}
									</div>
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

					<!-- Database Management Section -->
					<div class="mb-6">
						<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
							Database Management
						</h4>
						<div class="flex flex-wrap gap-2">
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
								on:click={() => executeRaindexQuery(() => RaindexClient.clearTables(queryFn))}
								disabled={isLoading}
								color="red"
								size="sm"
							>
								Clear All Tables & Views
							</Button>
						</div>
					</div>

					<!-- Orders Section -->
					<div class="mb-6">
						<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">Orders</h4>
						<div class="flex flex-wrap gap-2">
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
									if (showOrderHashInput && orderHashInputType === 'trades') {
										showOrderHashInput = false;
										orderHashInputType = '';
									} else {
										showOrderHashInput = true;
										showVaultHistoryInputs = false;
										orderHashInputType = 'trades';
									}
								}}
								disabled={isLoading}
								color="light"
								size="sm"
							>
								Get Trades for Order
							</Button>
							<Button
								on:click={() => {
									if (showOrderHashInput && orderHashInputType === 'volumes') {
										showOrderHashInput = false;
										orderHashInputType = '';
									} else {
										showOrderHashInput = true;
										showVaultHistoryInputs = false;
										orderHashInputType = 'volumes';
									}
								}}
								disabled={isLoading}
								color="light"
								size="sm"
							>
								Get Order Vault Volumes
							</Button>
						</div>
					</div>

					<!-- Vaults Section -->
					<div class="mb-6">
						<h4 class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">Vaults</h4>
						<div class="flex flex-wrap gap-2">
							<Button
								on:click={() => executeRaindexQuery(() => RaindexClient.getAllVaults(queryFn))}
								disabled={isLoading}
								color="light"
								size="sm"
							>
								Get All Vaults
							</Button>
							<Button
								on:click={() => {
									showVaultHistoryInputs = !showVaultHistoryInputs;
									showOrderHashInput = false;
								}}
								disabled={isLoading}
								color="light"
								size="sm"
							>
								Get Vault Balance History
							</Button>
						</div>
					</div>
				</div>

				<!-- Order Hash Input Form -->
				{#if showOrderHashInput}
					<div
						class="mb-4 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-600 dark:bg-blue-900/20"
					>
						<Label for="order-hash-input" class="mb-2 text-sm font-medium">Order Hash</Label>
						<Input
							id="order-hash-input"
							bind:value={orderHashInput}
							placeholder="Enter order hash (0x...)"
							class="mb-3"
						/>
						<div class="flex gap-2">
							{#if orderHashInputType === 'trades'}
								<Button
									on:click={() => {
										if (orderHashInput.trim()) {
											executeRaindexQuery(() =>
												RaindexClient.getOrderTrades(queryFn, orderHashInput.trim())
											);
										}
									}}
									disabled={isLoading || !orderHashInput.trim()}
									color="blue"
									size="sm"
								>
									Get Trades
								</Button>
							{:else if orderHashInputType === 'volumes'}
								<Button
									on:click={() => {
										if (orderHashInput.trim()) {
											executeRaindexQuery(() =>
												RaindexClient.getOrderVaultVolumes(queryFn, orderHashInput.trim())
											);
										}
									}}
									disabled={isLoading || !orderHashInput.trim()}
									color="blue"
									size="sm"
								>
									Get Vault Volumes
								</Button>
							{/if}
							<Button
								on:click={() => {
									showOrderHashInput = false;
									orderHashInput = '';
									orderHashInputType = '';
								}}
								color="light"
								size="sm"
							>
								Cancel
							</Button>
						</div>
					</div>
				{/if}

				<!-- Vault Balance History Input Form -->
				{#if showVaultHistoryInputs}
					<div
						class="mb-4 rounded-lg border border-green-200 bg-green-50 p-4 dark:border-green-600 dark:bg-green-900/20"
					>
						<div class="grid grid-cols-1 gap-3 md:grid-cols-2">
							<div>
								<Label for="vault-id-input" class="mb-2 text-sm font-medium">Vault ID</Label>
								<Input id="vault-id-input" bind:value={vaultIdInput} placeholder="Enter vault ID" />
							</div>
							<div>
								<Label for="token-address-input" class="mb-2 text-sm font-medium"
									>Token Address</Label
								>
								<Input
									id="token-address-input"
									bind:value={tokenAddressInput}
									placeholder="Enter token address (0x...)"
								/>
							</div>
						</div>
						<div class="mt-3 flex gap-2">
							<Button
								on:click={() => {
									if (vaultIdInput.trim() && tokenAddressInput.trim()) {
										executeRaindexQuery(() =>
											RaindexClient.getVaultBalanceHistory(
												queryFn,
												vaultIdInput.trim(),
												tokenAddressInput.trim()
											)
										);
									}
								}}
								disabled={isLoading || !vaultIdInput.trim() || !tokenAddressInput.trim()}
								color="green"
								size="sm"
							>
								Get Balance History
							</Button>
							<Button
								on:click={() => {
									showVaultHistoryInputs = false;
									vaultIdInput = '';
									tokenAddressInput = '';
								}}
								color="light"
								size="sm"
							>
								Cancel
							</Button>
						</div>
					</div>
				{/if}

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

			{#if syncStatus}
				<div
					class="rounded-lg border border-blue-300 bg-white p-6 shadow-sm dark:border-blue-600 dark:bg-gray-800"
				>
					<h3 class="mb-3 text-lg font-medium text-blue-800 dark:text-blue-200">Sync Status</h3>
					<div
						class="rounded-lg bg-blue-50 p-4 text-sm text-blue-700 dark:bg-blue-900/20 dark:text-blue-300"
					>
						{syncStatus}
					</div>
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
