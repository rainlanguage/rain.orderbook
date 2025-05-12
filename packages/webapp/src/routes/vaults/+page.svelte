<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import {
		hideZeroBalanceVaults,
		showMyItemsOnly,
		orderHash,
		resetActiveNetworkRef,
		resetActiveOrderbookRef,
		activeSubgraphs
	} from '$lib/stores/settings';

	const {
		activeOrderbook,
		subgraphUrl,
		settings,
		accounts,
		activeAccountsItems,
		activeOrderStatus,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		activeNetworkOrderbooks
	} = $page.data.stores;

	onMount(async () => {
		if (!$activeOrderbook) {
			await resetActiveNetworkRef(settings, activeNetworkRef);
			resetActiveOrderbookRef(activeNetworkOrderbooks, activeOrderbookRef);
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
	{activeOrderStatus}
	{hideZeroBalanceVaults}
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeAccounts}
/>
