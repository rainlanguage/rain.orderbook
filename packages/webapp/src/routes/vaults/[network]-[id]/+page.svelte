<script lang="ts">
	import { invalidateTanstackQueries, PageHeader, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';

	const queryClient = useQueryClient();
	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const { account } = useAccount();

	function onDeposit(vault: SgVault) {
		handleDepositModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
				},
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			}
		});
	}

	function onWithdraw(vault: SgVault) {
		handleWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
				},
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			}
		});
	}
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
	id={$page.params.id}
	network={$page.params.network}
	{lightweightChartsTheme}
	{settings}
	{activeNetworkRef}
	{activeOrderbookRef}
	{onDeposit}
	{onWithdraw}
/>
