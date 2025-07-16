<script lang="ts">
	import { WalletConnect } from '@rainlanguage/ui-components';
	import { Modal, Button } from 'flowbite-svelte';
	import { appKitModal, connected } from '$lib/stores/wagmi';
	import type { WithdrawMultipleModalProps } from '../services/handleMultipleVaultsWithdraw';
	import { formatUnits } from 'viem';

	/**
	 * Modal component for withdrawing tokens from a vault.
	 * This component should only be used for withdraw actions.
	 */
	export let open: WithdrawMultipleModalProps['open'];
	export let args: WithdrawMultipleModalProps['args'];
	export let onSubmit: WithdrawMultipleModalProps['onSubmit'];

	const { account } = args;

	function handleSubmit() {
		onSubmit();
		handleClose();
	}

	function handleClose() {
		open = false;
	}
</script>

<Modal bind:open autoclose={false} size="md">
	<div class="space-y-4">
		<h3 class="text-xl font-medium" data-testid="modal-title">Withdraw multiple vaults</h3>
		{#if !account}
			<div class="h-10">
				<p>Connect your wallet to continue.</p>
			</div>
		{/if}
		<div class="space-y-3">
			<div class="max-h-48 space-y-2 overflow-y-auto">
				{#each args.vaults as vault (vault.id)}
					<div class="flex flex-row items-start justify-between rounded-lg bg-gray-50 p-3">
						<span class="mr-2 truncate font-mono text-xs font-medium text-gray-900">{vault.id}</span
						>
						<span class="whitespace-nowrap text-sm font-semibold text-gray-900">
							{formatUnits(vault.balance, Number(vault.token.decimals ?? 18))}
							&nbsp;
							{vault.token.symbol}
						</span>
					</div>
				{/each}
			</div>
		</div>
		<div class="flex flex-col justify-end gap-2">
			<div class="flex gap-2">
				<Button color="alternative" on:click={handleClose}>Cancel</Button>
				{#if account}
					<div class="flex flex-col gap-2">
						<Button color="blue" data-testid="withdraw-button" on:click={handleSubmit}>
							Withdraw all
						</Button>
					</div>
				{:else}
					<WalletConnect {appKitModal} {connected} />
				{/if}
			</div>
		</div>
	</div>
</Modal>
