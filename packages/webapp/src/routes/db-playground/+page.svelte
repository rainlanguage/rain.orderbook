<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { PageHeader } from '@rainlanguage/ui-components';
	import { Button, Textarea } from 'flowbite-svelte';
	import init, { SQLiteWasmDatabase, type WasmEncodedResult } from 'sqlite-worker';

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
						<Button
							on:click={() =>
								executeCommonQuery("SELECT name FROM sqlite_master WHERE type='view';")}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Fetch All Views
						</Button>
						<Button
							on:click={() =>
								executeCommonQuery(`
									SELECT 
										'page_count' as metric, 
										(SELECT * FROM pragma_page_count) as value
									UNION ALL
									SELECT 
										'page_size' as metric, 
										(SELECT * FROM pragma_page_size) as value
									UNION ALL
									SELECT 
										'freelist_count' as metric, 
										(SELECT * FROM pragma_freelist_count) as value
									UNION ALL
									SELECT 
										'database_size_bytes' as metric, 
										(SELECT * FROM pragma_page_count) * (SELECT * FROM pragma_page_size) as value
									UNION ALL
									SELECT 
										'database_size_mb' as metric, 
										ROUND(CAST((SELECT * FROM pragma_page_count) * (SELECT * FROM pragma_page_size) AS REAL) / 1048576, 2) as value;
								`)}
							disabled={isLoading}
							color="light"
							size="sm"
						>
							Database Size Info
						</Button>
						<Button
							on:click={() =>
								executeCommonQuery(`
									-- Drop all views first (they depend on tables)
									DROP VIEW IF EXISTS v_order_complete;
									DROP VIEW IF EXISTS v_order_inputs;
									DROP VIEW IF EXISTS v_order_outputs;
									
									-- Drop all tables (order matters due to foreign keys)
									DROP TABLE IF EXISTS signed_context_values;
									DROP TABLE IF EXISTS signed_contexts;
									DROP TABLE IF EXISTS take_orders;
									DROP TABLE IF EXISTS order_ios;
									DROP TABLE IF EXISTS ios;
									DROP TABLE IF EXISTS clears;
									DROP TABLE IF EXISTS clear_configs;
									DROP TABLE IF EXISTS order_details;
									DROP TABLE IF EXISTS orders;
									DROP TABLE IF EXISTS evaluables;
									DROP TABLE IF EXISTS withdraws;
									DROP TABLE IF EXISTS deposits;
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
