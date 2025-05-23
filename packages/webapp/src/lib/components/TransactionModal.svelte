<script lang="ts">
	import { Modal, Spinner, Button } from 'flowbite-svelte';
	import { TransactionStatus } from '@rainlanguage/ui-components';
	import { transactionStore } from '@rainlanguage/ui-components';
	import { createEventDispatcher } from 'svelte';

	export let open: boolean;
	export let messages: {
		success: string;
		error?: string;
		pending: string;
	};

	function handleClose() {
		open = false;
	}
	const dispatch = createEventDispatcher();

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		dispatch('success');
	}
</script>

<Modal size="sm" class="bg-opacity-90 backdrop-blur-sm" bind:open data-testid="transaction-modal">
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
				>
					{$transactionStore.status}
				</p>
				<p
					class="w-full whitespace-pre-wrap break-words text-center font-normal text-gray-900 dark:text-white"
				>
					{$transactionStore.error}
				</p>
				<Button on:click={handleClose}>Dismiss</Button>
			{:else if $transactionStore.status === TransactionStatus.SUCCESS}
				<div
					class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900"
				>
					<h1 class="text-lg md:text-2xl">✅</h1>
				</div>
				<div class="flex flex-col gap-4 text-center">
					<div class="flex flex-col">
						<p
							class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
						>
							{messages.success}
						</p>
						{#if $transactionStore.message}
							<p
								class="w-full whitespace-pre-wrap break-words text-center text-sm font-normal text-gray-900 dark:text-white"
							>
								{$transactionStore.message}
							</p>
						{/if}
					</div>
					<div class="flex flex-col gap-2">
						<div class="flex flex-row justify-center gap-2">
							{#if $transactionStore.newOrderHash && $transactionStore.network}
								<a href={`/orders/${$transactionStore.network}-${$transactionStore.newOrderHash}`}>
									<Button on:click={handleClose} color="alternative">View order</Button>
								</a>
							{/if}
							<Button on:click={handleClose}>Dismiss</Button>
						</div>
					</div>
				</div>
			{:else}
				<div
					class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary-100 dark:bg-primary-900"
				>
					<Spinner color="blue" size={10} />
				</div>
				<p
					class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
				>
					{messages.pending}
				</p>
				<p
					class="w-full whitespace-pre-wrap break-words text-center font-normal text-gray-900 dark:text-white"
				>
					{$transactionStore.message}
				</p>
			{/if}
			{#if $transactionStore.explorerLink}
				<p>
					<a
						data-testid="explorer-link"
						class="cursor-pointer text-blue-500 hover:underline"
						rel="noopener noreferrer"
						href={$transactionStore.explorerLink}
						target="_blank"
					>
						View transaction on block explorer.
					</a>
				</p>
			{/if}
		</div>
	{/if}
</Modal>
