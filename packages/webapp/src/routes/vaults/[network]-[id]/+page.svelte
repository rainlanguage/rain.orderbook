<script lang="ts">
	import {
		invalidateTanstackQueries,
		PageHeader,
		useAccount,
		useTransactions
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import {
		handleDepositModal,
		handleTransactionConfirmationModal,
		handleWithdrawModal
	} from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { type SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { useToasts } from '@rainlanguage/ui-components';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';

	const queryClient = useQueryClient();
	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	function onDeposit(vault: SgVault) {
		handleDepositModal({
			open: true,
			args: {
				vault,
				onDeposit: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
				},
				chainId,
				rpcUrl,
				subgraphUrl,
				// Casting to Hex since the buttons cannot appear if account is null
				account: $account as Hex
			}
		});
	}
	async function onWithdraw(vault: SgVault) {
		await handleVaultWithdraw(vault, {
			handleWithdrawModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			orderbookAddress,
			subgraphUrl,
			chainId,
			account: $account as Hex,
			rpcUrl
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
