<script lang="ts">
	import { useRegistry } from '@rainlanguage/ui-components';
	import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';

	const { registry, loading, error, appendRegistryToHref } = useRegistry();

	let orders: Array<[string, NameAndDescriptionCfg]> = [];
	let listError: string | null = null;

	$: if ($registry) {
		const result = $registry.getAllOrderDetails();
		if (result.error) {
			listError = result.error.readableMsg ?? result.error.msg ?? 'Failed to load orders';
			orders = [];
		} else {
			const detailsMap: Map<string, NameAndDescriptionCfg> = result.value ?? new Map();
			orders = Array.from(detailsMap.entries());
			listError = null;
		}
	}
</script>

<div class="flex w-full max-w-6xl flex-col gap-y-6">
	<div class="text-4xl font-semibold text-gray-900 dark:text-white">Algorithmic Orders</div>

	<div class="flex flex-col rounded-3xl bg-primary-100 p-12 dark:bg-primary-900">
		<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
			Raindex empowers you to take full control of your trading orders. All the orders here are
			non-custodial, perpetual, and automated orders built with our open-source, DeFi-native
			language <a class="underline" target="_blank" href="https://rainlang.xyz">Rainlang</a>
		</h1>
	</div>
	{#if $loading}
		<div class="text-center text-lg">Loading orders…</div>
	{:else if $error}
		<div class="flex gap-2 text-lg">
			Failed to initialize registry:
			<span class="text-red-500" data-testid="error-message">{$error}</span>
		</div>
	{:else if listError}
		<div class="flex gap-2 text-lg">
			Failed to load orders:
			<span class="text-red-500" data-testid="error-message">{listError}</span>
		</div>
	{:else if orders.length === 0}
		<div class="text-center text-lg">No orders found</div>
	{:else}
		<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3" data-testid="valid-orders">
			{#each orders as [orderKey, details]}
				<a
					href={appendRegistryToHref(`/deploy/${orderKey}`)}
					data-testid="order-short-tile"
					class="flex flex-col gap-y-2 rounded-xl border border-gray-200 p-4 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-900"
				>
					<h3 class="text-2xl font-semibold text-gray-900 dark:text-white">{details.name}</h3>
					<p class="text-lg text-gray-600 dark:text-gray-400">{details.short_description}</p>
				</a>
			{/each}
		</div>
	{/if}
</div>
