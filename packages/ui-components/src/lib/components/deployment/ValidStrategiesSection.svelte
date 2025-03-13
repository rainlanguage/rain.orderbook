<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type { StrategyDetail } from '$lib/types/strategy';

	export let strategies: StrategyDetail[];
	let customRegistryParam = '';

	onMount(async () => {
		// Get the custom registry from URL if it exists
		customRegistryParam = $page.url.searchParams.get('registry')
			? `?registry=${$page.url.searchParams.get('registry')}`
			: '';
	});
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3" data-testid="valid-strategies">
	{#each strategies as strategy}
		<a
			href={`/deploy/${strategy.name}${customRegistryParam}`}
			data-testid="strategy-short-tile"
			class="flex flex-col gap-y-2 rounded-xl border border-gray-200 p-4 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-900"
		>
			<h3 class="text-2xl font-semibold text-gray-900 dark:text-white">
				{strategy.details.name}
			</h3>
			<p class="text-lg text-gray-600 dark:text-gray-400">
				{strategy.details.short_description}
			</p>
		</a>
	{/each}
</div>
