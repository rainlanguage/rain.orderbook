<script lang="ts">
	import { PageHeader, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { handleVaultAction } from '$lib/services/handleVaultAction';

	const queryClient = useQueryClient();
	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const rpcUrls = $settings?.networks?.[network]?.['rpcs'] || [];
	const { account } = useAccount();

	function onDeposit(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'deposit',
			queryClient,
			queryKey: $page.params.id,
			chainId,
			rpcUrls,
			subgraphUrl,
			account: $account as Hex
		});
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'withdraw',
			queryClient,
			queryKey: $page.params.id,
			chainId,
			rpcUrls,
			subgraphUrl,
			account: $account as Hex
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
