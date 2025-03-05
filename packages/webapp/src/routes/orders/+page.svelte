<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader, useWagmiClient } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import { writable } from 'svelte/store';

	const {
		activeSubgraphs,
		settings,
		accounts,
		activeAccountsItems,
		activeOrderStatus,
		orderHash,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		showMyItemsOnly = writable(false)
	}: AppStoresInterface = $page.data.stores;

	const { connected } = useWagmiClient();

	$: currentRoute = $page.url.pathname;
	$: showMyItemsOnly.set($connected);
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeSubgraphs}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showMyItemsOnly}
	{activeOrderStatus}
	{orderHash}
	{hideZeroBalanceVaults}
	{currentRoute}
/>
