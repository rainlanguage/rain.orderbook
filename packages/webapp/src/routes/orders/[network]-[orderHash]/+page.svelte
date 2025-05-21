<script lang="ts">
	import { OrderDetail, PageHeader, useAccount, useToasts } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import {
		handleDepositModal,
		handleWithdrawModal,
		handleTransactionConfirmationModal
	} from '$lib/services/modal';
	import {
		getRemoveOrderCalldata,
		getVaultApprovalCalldata,
		getVaultWithdrawCalldata,
		type SgOrder,
		type SgVault
	} from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
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
		let calldata: string;
		try {
			const calldataResult = await getRemoveOrderCalldata(order);
			if (calldataResult.error) {
				return errToast(calldataResult.error.msg);
			}
			calldata = calldataResult.value;
			handleTransactionConfirmationModal({
				open: true,
				args: {
					chainId,
					entity: order,
					orderbookAddress,
					onConfirm: (txHash: Hex) => {
						manager.createRemoveOrderTransaction({
							subgraphUrl,
							txHash,
							queryKey: orderHash,
							chainId,
							networkKey: network,
							entity: order
						});
					},
					calldata
				}
			});
		} catch {
			return errToast('Failed to get calldata for order removal.');
		}
	}

	function onDeposit(vault: SgVault) {
		handleDepositModal({
			open: true,
			args: {
				vault,
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			},
			onSubmit: async (amount: bigint) => {
				const transaction = await handleVaultDeposit(
					rpcUrl,
					subgraphUrl,
					network,
					chainId,
					vault,
					amount,
					manager,
					errToast
				);

				if (transaction) {
					handleTransactionConfirmationModal({
						open: true,
						args: {
							entity: vault,
							orderbookAddress,
							chainId,
							onConfirm: transaction.onConfirm,
							calldata: transaction.calldata
						}
					});
				}
			}
		});
	}

	async function onWithdraw(vault: SgVault) {
		handleWithdrawModal({
			open: true,
			args: {
				vault,
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account as Hex
			},
			onSubmit: async (amount: bigint) => {
				let calldata: string;
				try {
					const calldataResult = await getVaultWithdrawCalldata(vault, amount.toString());
					if (calldataResult.error) {
						return errToast(calldataResult.error.msg);
					}
					calldata = calldataResult.value;
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
									queryKey: vault.id,
									entity: vault
								});
							},
							calldata
						}
					});
				} catch {
					return errToast('Failed to get calldata for vault withdrawal.');
				}
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
