<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable } from '@rainlanguage/ui-components';
	import type { AppStoresInterface } from '@rainlanguage/ui-components';
	import { signerAddress } from '$lib/stores/wagmi';
	import { connected } from '$lib/stores/wagmi.ts';
	import { writable } from 'svelte/store';

	const {
		activeSubgraphs,
		settings,
		activeOrderStatus,
		orderHash,
		hideZeroBalanceVaults,
		activeNetworkRef,
		activeOrderbookRef,
		showMyItemsOnly = writable(false)
	}: AppStoresInterface = $page.data.stores;

	$: currentRoute = $page.url.pathname;
	$: showMyItemsOnly.set($connected);
</script>

<OrdersListTable
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeSubgraphs}
	{settings}
	{showMyItemsOnly}
	{signerAddress}
	{activeOrderStatus}
	{orderHash}
	{hideZeroBalanceVaults}
	{currentRoute}
/>
