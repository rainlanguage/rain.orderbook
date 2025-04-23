<script lang="ts">
	import { PageHeader, VaultsListTable, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { writable } from 'svelte/store';

	const { account } = useAccount();

	const {
		activeOrderbook,
		subgraphUrl,
		orderHash,
		activeSubgraphs,
		settings,
		accounts,
		activeAccountsItems,
		activeOrderStatus,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		showMyItemsOnly = writable(false)
	} = $page.data.stores;

	$: showMyItemsOnly.set($account);
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
	currentRoute={$page.url.pathname}
/>
