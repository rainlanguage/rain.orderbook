<script lang="ts">
	import type { LocalDbStatus } from '@rainlanguage/orderbook';
	import LocalDbStatusBadge from './LocalDbStatusBadge.svelte';

	export let status: LocalDbStatus = 'active';
	export let error: string | undefined = undefined;
	export let label = 'LocalDB';

	let copied = false;

	const copyError = async () => {
		if (!error || typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
			return;
		}
		await navigator.clipboard.writeText(error);
		copied = true;
	};
</script>

<svelte:element
	class="rounded-lg border border-gray-200 bg-white px-3 py-3 dark:border-gray-700 dark:bg-gray-900"
	data-testid="local-db-status-card"
>
	<div class="flex items-center justify-between">
		<span
			class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
		>
			{label}
		</span>
		<LocalDbStatusBadge {status} />
	</div>
	{#if error && status === 'failure'}
		<button
			class="mt-2 inline-flex w-full items-center justify-center rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-xs font-semibold uppercase tracking-wide text-gray-600 transition hover:bg-gray-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-gray-400 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700 dark:focus-visible:ring-gray-500"
			type="button"
			on:click={copyError}
			disabled={!error}
			data-testid="local-db-error-copy"
		>
			{copied ? 'Copied!' : 'Copy error details'}
		</button>
	{/if}
</svelte:element>
