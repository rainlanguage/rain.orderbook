<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import {
		hideZeroBalanceVaults,
		showMyItemsOnly,
		orderHash,
		activeTokens
	} from '$lib/stores/settings';
	import { selectedChainIds } from '$lib/stores/settings';

	const { settings, accounts, activeAccountsItems, showInactiveOrders } = $page.data.stores;
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{orderHash}
	{showMyItemsOnly}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showInactiveOrders}
	{hideZeroBalanceVaults}
	{activeTokens}
	{selectedChainIds}
/>
