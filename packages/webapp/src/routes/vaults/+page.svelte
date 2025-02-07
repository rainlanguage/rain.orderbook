<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';

	const {
		activeOrderbook,
		subgraphUrl,
		orderHash,
		activeAccountsItems,
		activeSubgraphs,
		settings,
		activeOrderStatus,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		walletAddressMatchesOrBlank,
		activeNetworkOrderbooks,
		showMyItemsOnly
	} = $page.data.stores;

	export async function resetActiveNetworkRef() {
		const $networks = $settings?.networks;

		if ($networks !== undefined && Object.keys($networks).length > 0) {
			activeNetworkRef.set(Object.keys($networks)[0]);
		} else {
			activeNetworkRef.set(undefined);
		}
	}

	export function resetActiveOrderbookRef() {
		const $activeNetworkOrderbookRefs = Object.keys($activeNetworkOrderbooks);

		if ($activeNetworkOrderbookRefs.length > 0) {
			activeOrderbookRef.set($activeNetworkOrderbookRefs[0]);
		} else {
			activeOrderbookRef.set(undefined);
		}
	}

	onMount(async () => {
		if (!$activeOrderbook) {
			await resetActiveNetworkRef();
			resetActiveOrderbookRef();
		}
	});
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{activeOrderbook}
	{subgraphUrl}
	{orderHash}
	{showMyItemsOnly}
	{activeAccountsItems}
	{activeSubgraphs}
	{settings}
	{activeOrderStatus}
	{hideZeroBalanceVaults}
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeAccounts}
	{walletAddressMatchesOrBlank}
	currentRoute={$page.url.pathname}
/>
