<script lang="ts">
	import {
		InputTokenAmount,
		WalletConnect,
		type VaultActionArgs
	} from '@rainlanguage/ui-components';
	import { Modal, Button } from 'flowbite-svelte';
	import { appKitModal, connected, wagmiConfig } from '$lib/stores/wagmi';
	import { readContract, switchChain } from '@wagmi/core';
	import { erc20Abi, formatUnits, type Hex } from 'viem';
	import * as allChains from 'viem/chains';
	import { validateAmount } from '$lib/services/validateAmount';
	import { fade } from 'svelte/transition';
	import truncateEthAddress from 'truncate-eth-address';

	const { ...chains } = allChains;

	function getTargetChain(chainId: number) {
		for (const chain of Object.values(chains)) {
			if (chain.id === chainId) {
				return chain;
			}
		}
		throw new Error(`Chain with id ${chainId} not found`);
	}

	/**
	 * Modal component for depositing or withdrawing tokens from a vault.
	 */
	export let open: boolean;
	export let args: VaultActionArgs;
	export let onSubmit: (amount: bigint) => void;
	export let actionType: 'deposit' | 'withdraw';

	const { vault, chainId, account } = args;

	let amount: bigint = 0n;
	let userBalance: bigint = 0n;
	let errorMessage = '';
	let isSubmitting = false;

	const getUserBalance = async () => {
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
		} catch (error) {
			errorMessage =
				error instanceof Error ? error.message : `Failed to get user balance for ${actionType}.`;
			return;
		}
		return userBalance;
	};

	async function handleSubmit() {
		isSubmitting = true;
		try {
			await onSubmit(amount);
		} catch (e) {
			errorMessage = e instanceof Error ? e.message : `Failed to ${actionType}.`;
		} finally {
			isSubmitting = false;
			handleClose();
		}
	}

	function handleClose() {
		open = false;
		amount = 0n;
		errorMessage = '';
	}

	$: validationSource = actionType === 'deposit' ? userBalance : BigInt(vault.balance);
	$: validation = validateAmount(amount, validationSource);
	$: maxValue = validationSource;
	$: modalTitle = actionType === 'deposit' ? 'Deposit' : 'Withdraw';
	$: submitButtonText = actionType === 'deposit' ? 'Deposit' : 'Withdraw';
	$: submitButtonTestId = actionType === 'deposit' ? 'deposit-button' : 'withdraw-button';
</script>

<Modal bind:open autoclose={false} size="md" on:close={handleClose}>
	<div class="space-y-4">
		<h3 class="text-xl font-medium" data-testid="modal-title">{modalTitle}</h3>

		<div class="h-10">
			{#if account}
				{#await getUserBalance() then balance}
					{#if balance !== undefined || userBalance === 0n}
						<!-- Check balance specifically -->
						<div in:fade class="w-full flex-col justify-between">
							<div class="flex justify-between">
								<p>
									Balance of connected wallet <span class="text-green-500"
										>{truncateEthAddress(account)}</span
									>
								</p>
								<p in:fade>
									{formatUnits(userBalance, Number(vault.token.decimals))}
									{vault.token.symbol}
								</p>
							</div>
							<div class="flex justify-between">
								<p>Balance of vault</p>
								<p in:fade>
									{formatUnits(BigInt(vault.balance), Number(vault.token.decimals))}
									{vault.token.symbol}
								</p>
							</div>
						</div>
					{/if}
				{:catch error}
					<p class="text-red-500">Error fetching balance: {error.message}</p>
				{/await}
			{:else}
				<p>Connect your wallet to continue.</p>
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
							data-testid={submitButtonTestId}
							on:click={handleSubmit}
							disabled={!validation.isValid || isSubmitting}
						>
							{#if isSubmitting}
								Submitting...
							{:else}
								{submitButtonText}
							{/if}
						</Button>
					</div>
				{:else}
					<WalletConnect {appKitModal} {connected} />
				{/if}
			</div>
			{#if errorMessage}
				<p data-testid="error-message" class="text-red-500">{errorMessage}</p>
			{/if}
			{#if validation.exceedsBalance}
				<p class="text-red-500" data-testid="amount-error">
					{validation.errorMessage}
				</p>
			{/if}
		</div>
	</div>
</Modal>
