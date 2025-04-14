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
	} from '@rainlanguage/orderbook';
	import { Modal, Button } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';
	import { appKitModal, connected, wagmiConfig } from '$lib/stores/wagmi';
	import { readContract, switchChain } from '@wagmi/core';
	import { erc20Abi, formatUnits, type Hex } from 'viem';
	import * as allChains from 'viem/chains';
	import { validateAmount } from '$lib/services/validateAmount';
	import { fade } from 'svelte/transition';

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

	const { action, vault, chainId, rpcUrl, subgraphUrl, account } = args;

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
		pending: 'Processing your transaction...'
	};

	const getUserBalance = async () => {
		if (action !== 'deposit') return;
		const targetChain = getTargetChain(chainId);
		try {
			await switchChain($wagmiConfig, { chainId });
		} catch {
			errorMessage = `Switch to ${targetChain.name} to check your balance.`;
			return;
		}
		try {
			userBalance = await readContract($wagmiConfig, {
				abi: erc20Abi,
				address: vault.token.address as Hex,
				functionName: 'balanceOf',
				args: [account as Hex]
			});
		} catch {
			errorMessage = 'Failed to get user balance.';
			return;
		}
		return userBalance;
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

	$: validation = validateAmount(
		amount,
		action === 'deposit' ? userBalance : BigInt(vault.balance)
	);

	$: maxValue = action === 'deposit' ? userBalance : BigInt(vault.balance);
</script>

{#if currentStep === 1}
	<Modal bind:open autoclose={false} size="md">
		<div class="space-y-4">
			<h3 class="text-xl font-medium">Enter Amount</h3>

			<div class="h-4">
				{#if action === 'deposit'}
					{#await getUserBalance() then userBalance}
						{#if userBalance || userBalance === 0n}
							<div in:fade>
								<span class="font-semibold"
									>{action === 'deposit' ? 'Account balance:' : 'Vault balance:'}</span
								>
								<span in:fade>{formatUnits(userBalance, Number(vault.token.decimals))}</span>
							</div>
						{/if}
					{/await}
				{:else}
					<span class="font-semibold">Vault Balance:</span>
					<span in:fade>{formatUnits(BigInt(vault.balance), Number(vault.token.decimals))}</span>
				{/if}
			</div>
			<InputTokenAmount
				bind:value={amount}
				symbol={vault.token.symbol}
				decimals={Number(vault.token.decimals)}
				{maxValue}
			/>
			<div class="flex flex-col justify-end gap-2">
				<div class="flex gap-2">
					<Button color="alternative" on:click={handleClose}>Cancel</Button>
					{#if account}
						<div class="flex flex-col gap-2">
							<Button
								color="blue"
								data-testid="deposit-withdraw-button"
								on:click={handleContinue}
								disabled={!validation.isValid || isCheckingCalldata}
							>
								{#if isCheckingCalldata}
									Checking...
								{:else}
									{action === 'deposit' ? 'Deposit' : 'Withdraw'}
								{/if}
							</Button>
						</div>
					{:else}
						<WalletConnect {appKitModal} {connected} />
					{/if}
				</div>
				{#if errorMessage}
					<p data-testid="error-message">{errorMessage}</p>
				{/if}
				{#if validation.exceedsBalance}
					<p class="text-red-500" data-testid="amount-error">
						{validation.errorMessage}
					</p>
				{/if}
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} />
{/if}
