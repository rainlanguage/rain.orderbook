<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import {
		orderHash,
		showMyItemsOnly,
		showInactiveOrders,
		activeTokens,
		selectedChainIds
	} from '$lib/stores/settings';

	const {
		activeAccountsItems,
		hideZeroBalanceVaults,
		hideInactiveOrdersVaults
	}: AppStoresInterface = $page.data.stores;
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{selectedChainIds}
	{activeAccountsItems}
	{showMyItemsOnly}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{hideInactiveOrdersVaults}
	{activeTokens}
/>
