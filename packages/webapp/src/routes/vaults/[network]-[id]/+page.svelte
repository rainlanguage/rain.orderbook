<script lang="ts">
	import { PageHeader, useAccount, useToasts, useTransactions } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import {
		handleDepositModal,
		handleTransactionConfirmationModal,
		handleWithdrawModal
	} from '$lib/services/modal';
	import { type SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const network = $page.params.network;
	const subgraphUrl = $settings.orderbook.subgraphs[network]?.url || '';
	const chainId = $settings.orderbook.networks[network]?.chainId || 0;
	const orderbookAddress = $settings.orderbook.orderbooks[network]?.address as Hex;
	const rpcUrls = $settings.orderbook.networks[network]?.rpcs || [];
	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onDeposit(vault: SgVault) {
		await handleVaultDeposit({
			vault,
			handleDepositModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			orderbookAddress,
			subgraphUrl,
			chainId,
			account: $account as Hex,
			rpcUrls
		});
	}

	async function onWithdraw(vault: SgVault) {
		await handleVaultWithdraw({
			vault,
			handleWithdrawModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			toAddress: orderbookAddress as Hex,
			subgraphUrl,
			chainId,
			account: $account as Hex,
			rpcUrls
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
