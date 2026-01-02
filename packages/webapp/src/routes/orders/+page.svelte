<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import {
		orderHash,
		showMyItemsOnly,
		showInactiveOrders,
		activeTokens,
		selectedChainIds,
		activeOrderbookAddresses
	} from '$lib/stores/settings';

	const { activeAccountsItems, hideZeroBalanceVaults }: AppStoresInterface = $page.data.stores;
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{selectedChainIds}
	{activeAccountsItems}
	{showMyItemsOnly}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{activeTokens}
	{activeOrderbookAddresses}
/>
