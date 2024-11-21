<script lang="ts">
	export let isVaultsPage = false;
	export let isOrdersPage = false;
</script>

/** eslint-disable svelte/valid-compile */ /** eslint-disable svelte/valid-compile */
<div data-testid="orderbook-filters">
	{#if isVaultsPage}
		<div data-testid="zero-balance-filter"></div>
	{/if}

	{#if isOrdersPage}
		<div data-testid="order-hash-filter"></div>
		<div data-testid="status-filter"></div>
	{/if}
	<div data-testid="accounts-filter"></div>
	<div data-testid="network-filter"></div>
</div>
