<script lang="ts">
	export let query: any;
	export let walletAddressMatchesOrBlank = undefined;
	export let handleOrderRemoveModal = undefined;
	export let currentRoute = '';

	$: isVaultsPage = currentRoute.startsWith('/vaults');
	$: isOrdersPage = currentRoute.startsWith('/orders');
</script>

<div data-testid="tanstack-table">
	<slot name="title" />
	<slot name="head" />
	<slot name="bodyRow" />

	{#if isVaultsPage}
		<div data-testid="vaults-content">Vaults Table Content</div>
	{/if}

	{#if isOrdersPage}
		<div data-testid="orders-content">Orders Table Content</div>
	{/if}

	{#if walletAddressMatchesOrBlank && handleOrderRemoveModal}
		<div data-testid="wallet-actions">Wallet Actions Content</div>
	{/if}
</div>
