<script lang="ts">
	import {
		invalidateTanstackQueries,
		PageHeader,
		VaultsListTable,
		useAccount
	} from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { hideZeroBalanceVaults, showMyItemsOnly, orderHash } from '$lib/stores/settings';
	import { activeSubgraphs } from '$lib/stores/settings';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import type { SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useQueryClient } from '@tanstack/svelte-query';

	const queryClient = useQueryClient();

	const {
		activeOrderbook,
		subgraphUrl,
		settings,
		accounts,
		activeAccountsItems,
		activeOrderStatus,
		activeNetworkRef,
		activeOrderbookRef,
		activeAccounts,
		activeNetworkOrderbooks
	} = $page.data.stores;

	const { account } = useAccount();
	const network = $page.params.network;
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';

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

	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
				},
				action,
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			}
		});
	}

	function onDeposit(vault: SgVault) {
		handleVaultAction(vault, 'deposit');
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction(vault, 'withdraw');
	}
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{activeOrderbook}
	{subgraphUrl}
	{orderHash}
	{showMyItemsOnly}
	{activeSubgraphs}
	{settings}
	{accounts}
	{activeAccountsItems}
	{activeOrderStatus}
	{hideZeroBalanceVaults}
	{activeNetworkRef}
	{activeOrderbookRef}
	{activeAccounts}
	{onDeposit}
	{onWithdraw}
/>
