<script lang="ts">
	import { OrderDetail, PageHeader, useAccount, useToasts } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleTransactionConfirmationModal, handleWithdrawModal } from '$lib/services/modal';
	import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	import { handleRemoveOrder } from '$lib/services/handleRemoveOrder';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
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

	function onDeposit(vault: SgVault) {
		handleVaultDeposit({
			vault,
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex
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
			orderbookAddress,
			subgraphUrl,
			chainId,
			account: $account as Hex,
			rpcUrl
		});
	}
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

<OrderDetail
	{orderHash}
	{subgraphUrl}
	{rpcUrl}
	{codeMirrorTheme}
	{lightweightChartsTheme}
	{colorTheme}
	{orderbookAddress}
	{onRemove}
	{onDeposit}
	{onWithdraw}
/>
