<script lang="ts">
	import {
		invalidateTanstackQueries,
		OrderDetail,
		PageHeader,
		useAccount,
		useToasts
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import {
		handleDepositOrWithdrawModal,
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
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onRemove(order: SgOrder) {
		let calldata;
		try {
			const calldataResult = await getRemoveOrderCalldata(order);
			if (calldataResult.error) {
				return errToast(calldataResult.error.msg);
			}
			calldata = calldataResult.value;
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
					calldata
				}
			});
		} catch {
			return errToast('Failed to get calldata for order removal.');
		}
	}

	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		const network = $page.params.network;
		const orderHash = $page.params.orderHash;
		const subgraphUrl = $settings?.subgraphs?.[network] || '';
		const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;

		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [orderHash]);
				},
				action,
				chainId,
				rpcUrl,
				subgraphUrl,
				// Casting to Hex since the buttons cannot appear if account is null
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
