<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import { connected } from '$lib/stores/wagmi.ts';
	import { writable } from 'svelte/store';
	import { activeSubgraphs, orderHash } from '$lib/stores/settings';

	const {
		settings,
		accounts,
		activeAccountsItems,
		activeOrderStatus,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		showMyItemsOnly = writable(false)
	}: AppStoresInterface = $page.data.stores;

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
