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
	 * Modal component for depositing tokens into a vault.
	 * This component should only be used for deposit actions.
	 */
	export let open: boolean;
	export let args: VaultActionArgs;
	export let onSubmit: (amount: bigint) => void;

	const { vault, chainId, account } = args;

	let amount: bigint = 0n;
	let userBalance: bigint = 0n;
	let errorMessage = '';
	let isCheckingCalldata = false;

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
		} catch {
			errorMessage = 'Failed to get user balance.';
			return;
		}
		return userBalance;
	};

	async function handleContinue() {
		onSubmit(amount);
		handleClose();
	}

	function handleClose() {
		open = false;
		amount = 0n;
	}

	$: validation = validateAmount(amount, userBalance);
	$: maxValue = userBalance;
</script>

<Modal bind:open autoclose={false} size="md">
	<div class="space-y-4">
		<h3 class="text-xl font-medium" data-testid="modal-title">Deposit</h3>

		<div class="h-10">
			{#if account}
				{#await getUserBalance() then userBalance}
					{#if userBalance || userBalance === 0n}
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
							data-testid="deposit-button"
							on:click={handleContinue}
							disabled={!validation.isValid || isCheckingCalldata}
						>
							{#if isCheckingCalldata}
								Checking...
							{:else}
								Deposit
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
