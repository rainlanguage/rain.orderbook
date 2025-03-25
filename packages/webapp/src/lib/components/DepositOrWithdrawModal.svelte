<script lang="ts">
	import {
		transactionStore,
		InputTokenAmount,
		WalletConnect,
		type DepositOrWithdrawArgs
	} from '@rainlanguage/ui-components';
	import {
		getVaultDepositCalldata,
		getVaultApprovalCalldata,
		type DepositCalldataResult,
		type WithdrawCalldataResult,
		type ApprovalCalldata,
		getVaultWithdrawCalldata
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';
	import { readContract, switchChain } from '@wagmi/core';
	import { erc20Abi, type Hex } from 'viem';
	import * as allChains from 'viem/chains';

	const { ...chains } = allChains;

	function getTargetChain(chainId: number) {
		for (const chain of Object.values(chains)) {
			if (chain.id === chainId) {
				return chain;
			}
		}
		throw new Error(`Chain with id ${chainId} not found`);
	}

	export let open: boolean;
	export let args: DepositOrWithdrawArgs;

	const { action, vault, chainId, rpcUrl, subgraphUrl } = args;

	type Action = 'deposit' | 'withdraw';
	const actionType = action as Action;

	let currentStep = 1;
	let amount: bigint = 0n;
	let userBalance: bigint = 0n;
	let errorMessage = '';
	let depositCalldata: DepositCalldataResult | undefined = undefined;
	let approvalCalldata: ApprovalCalldata | undefined = undefined;
	let withdrawCalldata: WithdrawCalldataResult | undefined = undefined;
	let isCheckingCalldata = false;

	const messages = {
		success: 'Transaction successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	$: if ($signerAddress && action === 'deposit') {
		getUserBalance();
	}

	const getUserBalance = async () => {
		const targetChain = getTargetChain(chainId);
		try {
			await switchChain($wagmiConfig, { chainId });
		} catch {
			return (errorMessage = `Switch to ${targetChain.name} to check your balance.`);
		}
		userBalance = await readContract($wagmiConfig, {
			abi: erc20Abi,
			address: vault.token.address as Hex,
			functionName: 'balanceOf',
			args: [$signerAddress as Hex]
		});
	};

	async function handleTransaction(
		transactionCalldata: DepositCalldataResult | WithdrawCalldataResult,
		approvalCalldata?: ApprovalCalldata | undefined
	) {
		transactionStore.handleDepositOrWithdrawTransaction({
			config: $wagmiConfig,
			transactionCalldata,
			approvalCalldata,
			action,
			chainId,
			vault,
			subgraphUrl
		});
	}

	async function handleContinue() {
		isCheckingCalldata = true;
		try {
			if (action === 'deposit') {
				try {
					approvalCalldata = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
				} catch {
					approvalCalldata = undefined;
				}
				depositCalldata = await getVaultDepositCalldata(vault, amount.toString());
				if (depositCalldata) {
					handleTransaction(depositCalldata, approvalCalldata);
				}
			} else if (action === 'withdraw') {
				withdrawCalldata = await getVaultWithdrawCalldata(vault, amount.toString());
				if (withdrawCalldata) {
					handleTransaction(withdrawCalldata);
				}
			}
			currentStep = 2;
		} catch {
			errorMessage = 'Failed to get calldata.';
		} finally {
			isCheckingCalldata = false;
		}
	}

	function handleClose() {
		transactionStore.reset();
		open = false;
		currentStep = 1;
		amount = 0n;
	}

	$: amountGreaterThanBalance = {
		deposit: amount > userBalance,
		withdraw: amount > BigInt(vault.balance)
	};
</script>

{#if currentStep === 1}
	<Modal bind:open autoclose={false} size="md">
		<div class="space-y-6">
			<div class="flex flex-col gap-4">
				<h3 class="text-xl font-medium">Enter Amount</h3>
			</div>
			<InputTokenAmount
				bind:value={amount}
				symbol={vault.token.symbol}
				decimals={Number(vault.token.decimals)}
				maxValue={action === 'deposit' ? userBalance : BigInt(vault.balance)}
			/>
			<div class="flex flex-col justify-end gap-2">
				<div class="flex gap-2">
					<Button color="alternative" on:click={handleClose}>Cancel</Button>
					{#if $signerAddress}
						<div class="flex flex-col gap-2">
							<Button
								color="blue"
								on:click={handleContinue}
								disabled={amount <= 0n ||
									amountGreaterThanBalance[actionType] ||
									isCheckingCalldata}
							>
								{#if isCheckingCalldata}
									Checking...
								{:else}
									{action === 'deposit' ? 'Deposit' : 'Withdraw'}
								{/if}
							</Button>
						</div>
					{:else}
						<WalletConnect {appKitModal} {connected} {signerAddress} />
					{/if}
				</div>
				{#if errorMessage}
					<p data-testid="error-message">{errorMessage}</p>
				{/if}
				{#if amountGreaterThanBalance[actionType]}
					<p class="text-red-500" data-testid="error">Amount cannot exceed available balance.</p>
				{/if}
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} />
{/if}
