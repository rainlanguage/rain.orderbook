<script lang="ts">
	import { onMount } from 'svelte';
	import { type NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';
	import { page } from '$app/stores';

	export let strategyDetails: NameAndDescriptionCfg;
	export let registryName: string;
	let customRegistryParam = '';

	onMount(async () => {
		// Get the custom registry from URL if it exists
		customRegistryParam = $page.url.searchParams.get('registry')
			? `?registry=${$page.url.searchParams.get('registry')}`
			: '';
	});
</script>

<a
	href={`/deploy/${registryName}${customRegistryParam}`}
	data-testid="strategy-short-tile"
	class="flex flex-col gap-y-2 rounded-xl border border-gray-200 p-4 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-900"
>
	<h3 class="text-2xl font-semibold text-gray-900 dark:text-white">{strategyDetails.name}</h3>
	<p class="text-lg text-gray-600 dark:text-gray-400">{strategyDetails.short_description}</p>
</a>
