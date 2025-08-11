<script lang="ts">
	import { Modal, Button } from 'flowbite-svelte';
	import { toHex } from 'viem';
	import type { WithdrawAllModalProps } from '$lib/services/handleVaultsWithdrawAll';

	/**
	 * Modal component for withdrawing tokens from multiple vaults.
	 * This component should only be used for batch withdraw actions.
	 */
	export let open: WithdrawAllModalProps['open'];
	export let vaults: WithdrawAllModalProps['vaults'];
	export let onSubmit: WithdrawAllModalProps['onSubmit'];

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
		<div class="space-y-3">
			<div class="max-h-48 space-y-2 overflow-y-auto">
				{#each vaults as vault (vault.id)}
					<div class="flex flex-row items-start justify-between rounded-lg bg-gray-50 p-3">
						<span class="mr-2 truncate font-mono text-xs font-medium text-gray-900"
							>{toHex(vault.vaultId)}</span
						>
						<span class="whitespace-nowrap text-sm font-semibold text-gray-900">
							{vault.formattedBalance}
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
				<Button color="blue" data-testid="withdraw-button" on:click={handleSubmit}>
					Withdraw all
				</Button>
			</div>
		</div>
	</div>
</Modal>
