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
	import {
		getRemoveOrderCalldata,
		getVaultWithdrawCalldata,
		type SgOrder,
		type SgVault
	} from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const { account } = useAccount();
	const { manager } = useTransactions();
	const queryClient = useQueryClient();

	function onRemove(order: SgOrder) {
		handleTransactionConfirmationModal({
			open: true,
			args: {
				entity: order,
				orderbookAddress,
				chainId,
				onConfirm: (txHash: Hex) => {
					manager.createRemoveOrderTransaction({
						subgraphUrl,
						txHash,
						chainId,
						networkKey: network,
						queryKey: orderHash
					});
				},
				getCalldataFn: () => getRemoveOrderCalldata(order)
			}
		});
	}

	function onDeposit(vault: SgVault) {
		handleDepositModal({
			open: true,
			args: {
				vault,
				onDeposit: () => {
					invalidateTanstackQueries(queryClient, [orderHash]);
				},
				chainId,
				rpcUrl,
				subgraphUrl,
				// Casting to Hex since the buttons cannot appear if account is null
				account: $account as Hex
			}
		});
	}

	function onWithdraw(vault: SgVault) {
		handleWithdrawModal({
			open: true,
			args: {
				vault,
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			},
			onSubmit: (amount: bigint) => {
				handleTransactionConfirmationModal({
					open: true,
					args: {
						entity: vault,
						orderbookAddress,
						chainId,
						onConfirm: (txHash: Hex) => {
							manager.createWithdrawTransaction({
								subgraphUrl,
								txHash,
								chainId,
								networkKey: network,
								queryKey: vault.id
							});
						},
						getCalldataFn: () => getVaultWithdrawCalldata(vault, amount.toString())
					}
				});
			}
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
