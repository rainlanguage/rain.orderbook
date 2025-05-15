<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import {
		activeSubgraphs,
		orderHash,
		showMyItemsOnly,
		showInactiveOrders
	} from '$lib/stores/settings';

	const {
		settings,
		accounts,
		activeAccountsItems,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef
	}: AppStoresInterface = $page.data.stores;
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
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
/>
