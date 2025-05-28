<script lang="ts">
	import { PageHeader, useAccount, useToasts, useTransactions } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { type SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onDeposit(vault: SgVault) {
		await handleVaultDeposit({
			vault,
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

	async function onWithdraw(vault: SgVault) {
		await handleVaultWithdraw({
			vault,
			errToast,
			manager,
			network,
			toAddress: orderbookAddress as Hex,
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
