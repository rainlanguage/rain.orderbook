<script lang="ts">
	import { PageHeader, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { handleVaultAction } from '$lib/services/vaultActions';

	const queryClient = useQueryClient();
	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const { account } = useAccount();

	function onDeposit(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'deposit',
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex,
			queryClient,
			vaultId: $page.params.id
		});
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'withdraw',
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex,
			queryClient,
			vaultId: $page.params.id
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
