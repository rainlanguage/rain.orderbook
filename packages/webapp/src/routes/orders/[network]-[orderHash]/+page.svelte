<script lang="ts">
	import {
		invalidateTanstackQueries,
		OrderDetail,
		PageHeader,
		useAccount
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import {
		handleDepositModal,
		handleWithdrawModal,
		handleTransactionConfirmationModal
	} from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { getRemoveOrderCalldata, type SgOrder, type SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const { account } = useAccount();
	const { manager } = useTransactions();

	function onRemove(order: SgOrder) {
		handleTransactionConfirmationModal({
			open: true,
			args: {
				order,
				orderbookAddress,
				chainId,
				onConfirm: (txHash: Hex) => {
					manager.createRemoveOrderTransaction({
						subgraphUrl,
						txHash,
						orderHash,
						chainId,
						networkKey: network
					});
				},
				getCalldataFn: () => getRemoveOrderCalldata(order)
			}
		});
	}

	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		const modalHandler = action === 'deposit' ? handleDepositModal : handleWithdrawModal;
		modalHandler({
			open: true,
			args: {
				vault,
				onSuccess: () => {
					invalidateTanstackQueries(queryClient, [$page.params.orderHash]);
				},
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
