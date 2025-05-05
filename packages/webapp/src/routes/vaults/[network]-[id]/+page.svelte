<script lang="ts">
	import {
		invalidateTanstackQueries,
		PageHeader,
		useAccount,
		useToasts
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
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
	const { addToast } = useToasts();
	const { account } = useAccount();
	$: console.log('account', subgraphUrl);
	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
					addToast({ message: 'Vault balance updated', type: 'success', color: 'green' });
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
