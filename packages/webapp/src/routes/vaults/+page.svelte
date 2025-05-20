<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { hideZeroBalanceVaults, showMyItemsOnly, orderHash } from '$lib/stores/settings';
	import { activeSubgraphs } from '$lib/stores/settings';
	import { resetActiveNetworkRef } from '$lib/services/resetActiveNetworkRef';
	import { resetActiveOrderbookRef } from '$lib/services/resetActiveOrderbookRef';

	const {
		activeOrderbook,
		subgraphUrl,
		settings,
		accounts,
		activeAccountsItems,
		showInactiveOrders,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		activeNetworkOrderbooks
	} = $page.data.stores;

	onMount(async () => {
		if (!$activeOrderbook) {
			await resetActiveNetworkRef(activeNetworkRef, settings);
			resetActiveOrderbookRef(activeOrderbookRef, activeNetworkOrderbooks);
		}
	});
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{activeOrderbook}
	{subgraphUrl}
	{orderHash}
	{showMyItemsOnly}
	{activeSubgraphs}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showInactiveOrders}
	{hideZeroBalanceVaults}
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeAccounts}
/>
