<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import {
		orderHash,
		showInactiveOrders,
		activeTokens,
		selectedChainIds,
		activeOrderbookAddresses,
		ownerFilter
	} from '$lib/stores/settings';

	const { hideZeroBalanceVaults, hideInactiveOrdersVaults }: AppStoresInterface = $page.data.stores;
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{selectedChainIds}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{hideInactiveOrdersVaults}
	{activeTokens}
	{activeOrderbookAddresses}
	{ownerFilter}
/>
