<script lang="ts">
	import { page } from '$app/stores';
	import { PageHeader, useLocalDb } from '@rainlanguage/ui-components';
	import { Button, Textarea } from 'flowbite-svelte';

	const localDb = useLocalDb();

	let sqlQuery = '';
	let queryResults: unknown = null;
	let isLoading = false;
	let error = '';
	let queryTime = 0;

	async function executeQuery() {
		isLoading = true;
		error = '';
		queryResults = null;
		queryTime = 0;

		const startTime = performance.now();
		const result = await localDb.query(sqlQuery);
		queryTime = performance.now() - startTime;

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
</script>

<PageHeader title="Database Playground" pathname={$page.url.pathname} />

<div class="mx-auto max-w-6xl space-y-6 p-4">
	<div
		class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800"
	>
		<h3 class="mb-4 text-lg font-medium text-gray-900 dark:text-gray-100">Database Operations</h3>

		<!-- Custom SQL Query Section -->
		<div class="mb-4">
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

	{#if queryTime > 0}
		<div
			class="rounded-lg border border-blue-300 bg-blue-50 p-4 shadow-sm dark:border-blue-600 dark:bg-blue-900/20"
		>
			<span class="font-medium text-blue-800 dark:text-blue-200">
				Query executed in {queryTime.toFixed(2)}ms
			</span>
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
