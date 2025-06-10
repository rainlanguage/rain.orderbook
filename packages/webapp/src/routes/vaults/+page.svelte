<script lang="ts">
	import { PageHeader, VaultsListTable, useToasts } from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { hideZeroBalanceVaults, showMyItemsOnly, orderHash } from '$lib/stores/settings';
	import { activeSubgraphs } from '$lib/stores/settings';
	import { resetActiveNetworkRef } from '$lib/services/resetActiveNetworkRef';
	import { resetActiveOrderbookRef } from '$lib/services/resetActiveOrderbookRef';

	const { errToast } = useToasts();

	const {
		activeOrderbook,
		settings,
		accounts,
		activeAccountsItems,
		showInactiveOrders,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		activeNetworkOrderbooks
	} = $page.data.stores;

	onMount(async () => {
		if (!$activeOrderbook) {
			try {
				resetActiveNetworkRef(activeNetworkRef, settings);
			} catch (error) {
				errToast((error as Error).message);
			}
			try {
				resetActiveOrderbookRef(activeOrderbookRef, activeNetworkOrderbooks);
			} catch (error) {
				errToast((error as Error).message);
			}
		}
	});
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{activeOrderbook}
	{orderHash}
	{showMyItemsOnly}
	{activeSubgraphs}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showInactiveOrders}
	{hideZeroBalanceVaults}
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeAccounts}
/>
