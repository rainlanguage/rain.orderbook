<script lang="ts">
	import { Modal, Spinner, Button } from 'flowbite-svelte';
	import { transactionStore, TransactionStatus } from '@rainlanguage/ui-components';
	export let open;
	const handleClose = () => {
		return transactionStore.reset();
	};
</script>

<Modal
	size="sm"
	class="bg-opacity-90 backdrop-blur-sm"
	{open}
	on:close={() => handleClose()}
	data-testid="transaction-modal"
>
	{#if $transactionStore.status !== TransactionStatus.IDLE}
		<div class="flex flex-col items-center justify-center gap-2 p-4">
			{#if $transactionStore.status === TransactionStatus.ERROR}
				<div
					class="mb-4 flex h-16 w-16 items-center justify-center rounded-full border-2 border-red-400 bg-red-100 dark:bg-red-900"
					data-testid="error-icon"
				>
					<h1 class="text-lg md:text-2xl">❌</h1>
				</div>
				<p
					class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
					data-testid="error-status"
				>
					{$transactionStore.status}
				</p>
				<p
					class="w-full whitespace-pre-wrap break-words text-center font-normal text-gray-900 dark:text-white"
					data-testid="error-message"
				>
					{$transactionStore.error}
				</p>

				<Button on:click={() => handleClose()} class="mt-4" data-testid="dismiss-button"
					>DISMISS</Button
				>
			{:else if $transactionStore.status === TransactionStatus.SUCCESS}
				<div
					class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900"
					data-testid="success-icon"
				>
					<h1 class="text-lg md:text-2xl">✅</h1>
				</div>
				<div class="flex flex-col gap-4 text-center">
					<p
						class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
						data-testid="success-status"
					>
						{$transactionStore.status}
					</p>
					{#if $transactionStore.message}
						<p
							class="w-full whitespace-pre-wrap break-words text-center text-sm font-normal text-gray-900 dark:text-white"
							data-testid="success-message"
						>
							{$transactionStore.message}
						</p>
					{/if}
				</div>

				<Button on:click={() => handleClose()} class="mt-4" data-testid="dismiss-button"
					>DISMISS</Button
				>
			{:else if $transactionStore.status === TransactionStatus.CHECKING_ALLOWANCE || $transactionStore.status === TransactionStatus.PENDING_WALLET || $transactionStore.status === TransactionStatus.PENDING_DEPLOYMENT || $transactionStore.status === TransactionStatus.PENDING_APPROVAL}
				<div
					class="bg-primary-100 dark:bg-primary-900 mb-4 flex h-16 w-16 items-center justify-center rounded-full"
					data-testid="spinner"
				>
					<Spinner color="blue" size={10} />
				</div>
				<p
					class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
					data-testid="pending-message"
				>
					{$transactionStore.message || $transactionStore.status}
				</p>
			{/if}
		</div>
	{/if}
</Modal>
