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

	const { settings, accounts, activeAccountsItems, hideZeroBalanceVaults }: AppStoresInterface =
		$page.data.stores;
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable
	{selectedChainIds}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showMyItemsOnly}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{activeTokens}
/>
