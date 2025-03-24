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
		type ApprovalCalldata
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button, Badge } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';
	import { readContract, switchChain } from '@wagmi/core';
	import { erc20Abi, formatUnits, type Hex } from 'viem';
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

	const { vault, chainId, rpcUrl, subgraphUrl } = args;

	let currentStep = 1;
	let amount: bigint = 0n;
	let userBalance: bigint = 0n;
	let errorMessage = '';
	let depositCalldata: DepositCalldataResult | undefined = undefined;
	let approvalCalldata: ApprovalCalldata | undefined = undefined;
	let isCheckingCalldata = false;

	const messages = {
		success: 'Transaction successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	$: if ($signerAddress) {
		getUserBalance();
	}

	const getUserBalance = async () => {
		const targetChain = getTargetChain(chainId);
		try {
			await switchChain($wagmiConfig, { chainId });
		} catch (error) {
			throw new Error(`Switch to ${targetChain.name} to check your balance.`);
		}

		userBalance = await readContract($wagmiConfig, {
			abi: erc20Abi,
			address: vault.token.address as Hex,
			functionName: 'balanceOf',
			args: [$signerAddress as Hex]
		});
		return userBalance;
	};

	async function handleTransaction(
		transactionCalldata: DepositCalldataResult,
		approvalCalldata?: ApprovalCalldata | undefined
	) {
		transactionStore.handleDepositOrWithdrawTransaction({
			config: $wagmiConfig,
			transactionCalldata,
			approvalCalldata,
			action: 'deposit',
			chainId,
			vault,
			subgraphUrl
		});
	}

	async function handleContinue() {
		isCheckingCalldata = true;
		try {
			try {
				approvalCalldata = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
			} catch {
				approvalCalldata = undefined;
			}
			depositCalldata = await getVaultDepositCalldata(vault, amount.toString());
			if (depositCalldata) {
				handleTransaction(depositCalldata, approvalCalldata);
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

	$: amountGreaterThanBalance = amount > userBalance;
</script>

{#if currentStep === 1}
	<Modal bind:open autoclose={false} size="md">
		<div class="space-y-6">
			<div class="flex flex-col gap-4">
				<h3 class="text-xl font-medium">Enter Deposit Amount</h3>
			</div>
			<div class="flex flex-col gap-2">
				<Badge color="yellow" class="w-fit" data-testid="balance-badge">
					{#await getUserBalance()}
						Loading your balance...
					{:then balance}
						Your balance: {formatUnits(balance, Number(vault.token.decimals))}
						{vault.token.symbol}
					{:catch error}
						Error loading balance: {error.message}
					{/await}
				</Badge>

				<InputTokenAmount
					bind:value={amount}
					symbol={vault.token.symbol}
					decimals={Number(vault.token.decimals)}
					maxValue={userBalance}
				/>
			</div>
			<div class="flex flex-col justify-end gap-2">
				<div class="flex gap-2">
					<Button color="alternative" on:click={handleClose}>Cancel</Button>
					{#if $signerAddress}
						<div class="flex flex-col gap-2">
							<Button
								color="blue"
								on:click={handleContinue}
								disabled={amount <= 0n || amountGreaterThanBalance || isCheckingCalldata}
							>
								{#if isCheckingCalldata}
									Checking...
								{:else}
									Deposit
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
				{#if amountGreaterThanBalance}
					<p class="text-red-500" data-testid="error">Amount cannot exceed available balance.</p>
				{/if}
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} />
{/if}
