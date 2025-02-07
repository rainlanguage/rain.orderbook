<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';

	const {
		activeSubgraphs,
		settings,
		activeAccountsItems,
		activeOrderStatus,
		orderHash,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		showMyItemsOnly
	}: AppStoresInterface = $page.data.stores;

	$: currentRoute = $page.url.pathname;
</script>

<OrdersListTable
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeSubgraphs}
	{settings}
	{showMyItemsOnly}
	{activeAccountsItems}
	{activeOrderStatus}
	{orderHash}
	{hideZeroBalanceVaults}
	{currentRoute}
/>
