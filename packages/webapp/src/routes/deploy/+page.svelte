<script lang="ts">
	import { ValidOrdersSection, InvalidOrdersSection } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	const { validOrders, invalidOrders, error } = $page.data;
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
	{#if error}
		<div class="flex gap-2 text-lg">
			Failed to load orders:<span class="text-red-500" data-testid="error-message">{error}</span>
		</div>
	{:else if validOrders.length === 0 && invalidOrders.length === 0}
		<div class="text-center text-lg">No orders found</div>
	{:else}
		{#if validOrders.length > 0}
			<ValidOrdersSection orders={validOrders} />
		{/if}
		{#if invalidOrders.length > 0}
			<InvalidOrdersSection ordersWithErrors={invalidOrders} />
		{/if}
	{/if}
</div>
