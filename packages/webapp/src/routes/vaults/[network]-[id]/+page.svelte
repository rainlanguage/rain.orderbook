<script lang="ts">
	import {
		invalidateTanstackQueries,
		PageHeader,
		useAccount,
		useToasts,
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
	import {
		getVaultApprovalCalldata,
		getVaultDepositCalldata,
		getVaultWithdrawCalldata,
		type SgVault
	} from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';

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

	async function handleDeposit(vault: SgVault, amount: bigint) {
		const calldata = await getVaultDepositCalldata(vault, amount.toString());
		if (calldata.error) {
			return errToast(calldata.error.msg);
		} else if (calldata.value) {
			handleTransactionConfirmationModal({
				open: true,
				args: {
					entity: vault,
					toAddress: orderbookAddress as Hex,
					chainId,
					onConfirm: (txHash: Hex) => {
						manager.createDepositTransaction({
							subgraphUrl,
							txHash,
							chainId,
							networkKey: network,
							queryKey: vault.id,
							entity: vault
						});
					},
					calldata: calldata.value
				}
			});
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
				const approvalResult = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
				if (approvalResult.error) {
					// If getting approval calldata fails, immediately invoke deposit
					handleDeposit(vault, amount);
				} else if (approvalResult.value) {
					handleTransactionConfirmationModal({
						open: true,
						args: {
							entity: vault,
							toAddress: vault.token.address as Hex,
							chainId,
							onConfirm: (txHash: Hex) => {
								manager.createApprovalTransaction({
									subgraphUrl,
									txHash,
									chainId,
									networkKey: network,
									queryKey: vault.id,
									entity: vault
								});
								// Immediately invoke deposit after approval
								handleDeposit(vault, amount);
							},
							calldata: approvalResult.value
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
							toAddress: orderbookAddress as Hex,
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
