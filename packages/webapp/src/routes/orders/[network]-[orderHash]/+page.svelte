<script lang="ts">
	import { OrderDetail, PageHeader, useAccount, useToasts } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import {
		handleDepositModal,
		handleTransactionConfirmationModal,
		handleWithdrawModal
	} from '$lib/services/modal';
	import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	import { handleRemoveOrder } from '$lib/services/handleRemoveOrder';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { orderHash, network } = $page.params;

	const { settings } = $page.data.stores;
	const orderbookAddress = $settings.orderbook.orderbooks[network]?.address || '';
	const subgraphUrl = $settings.orderbook.subgraphs[network]?.url || '';
	const rpcUrls = $settings.orderbook.networks[network]?.rpcs || [];
	const chainId = $settings.orderbook.networks[network]?.chainId || 0;
	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onRemove(order: SgOrder) {
		await handleRemoveOrder(order, {
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			orderbookAddress: orderbookAddress as Hex,
			subgraphUrl,
			chainId,
			orderHash
		});
	}

	async function onDeposit(vault: SgVault) {
		await handleVaultDeposit({
			vault,
			handleDepositModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			orderbookAddress: orderbookAddress as Hex,
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

<PageHeader title="Order" pathname={$page.url.pathname} />

<OrderDetail
	{orderHash}
	{subgraphUrl}
	{rpcUrls}
	{codeMirrorTheme}
	{lightweightChartsTheme}
	{colorTheme}
	{orderbookAddress}
	{onRemove}
	{onDeposit}
	{onWithdraw}
/>
