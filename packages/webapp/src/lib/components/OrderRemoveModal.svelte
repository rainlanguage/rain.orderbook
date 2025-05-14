<script lang="ts">
	import { type OrderRemoveArgs } from '@rainlanguage/ui-components';
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { RemoveOrder } from '@rainlanguage/ui-components';
	import { Modal, Spinner, Button } from 'flowbite-svelte';
	import { TransactionStatusMessage } from '@rainlanguage/ui-components';

	export let open: boolean = false;
	export let args: OrderRemoveArgs;
	let transaction: RemoveOrder;

	function handleClose() {
		open = false;
	}

	async function handleTransaction() {
		const removeOrderCalldata = await getRemoveOrderCalldata(args.order);
		transaction = new RemoveOrder({
			config: $wagmiConfig,
			...args,
			removeOrderCalldata
		});
		transaction.execute();
		open = true;
	}
	handleTransaction();
</script>

{#if transaction}
	<Modal size="sm" class="bg-opacity-90 backdrop-blur-sm" bind:open data-testid="transaction-modal">
		HI!!
		{#if transaction.state.status !== TransactionStatusMessage.IDLE}
			<div class="flex flex-col items-center justify-center gap-2 p-4">
				{#if 'errorDetails' in transaction.state}
					<div
						class="mb-4 flex h-16 w-16 items-center justify-center rounded-full border-2 border-red-400 bg-red-100 dark:bg-red-900"
						data-testid="error-icon"
					>
						<h1 class="text-lg md:text-2xl">❌</h1>
					</div>
					<p
						class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
					>
						{transaction.state.status}
					</p>
					<p
						class="w-full whitespace-pre-wrap break-words text-center font-normal text-gray-900 dark:text-white"
					>
						{transaction.state.errorDetails}
					</p>
					<Button on:click={handleClose}>Dismiss</Button>
				{:else}
					<div
						class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900"
					>
						{#if transaction.state.status === TransactionStatusMessage.SUCCESS}
							<h1 class="text-lg md:text-2xl">✅</h1>
						{:else}
							<Spinner color="blue" size={10} />
						{/if}
					</div>
					<div class="flex flex-col gap-4 text-center">
						<div class="flex flex-col">
							<p
								class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
							>
								{transaction.state.message}
							</p>
							<p
								class="w-full whitespace-pre-wrap break-words text-center text-sm font-normal text-gray-900 dark:text-white"
							>
								{transaction.state.status}
							</p>
						</div>
					</div>
				{/if}
				{#if 'explorerLink' in transaction.state}
					<p>
						<a
							data-testid="explorer-link"
							class="cursor-pointer text-blue-500 hover:underline"
							rel="noopener noreferrer"
							href={transaction.state.explorerLink}
							target="_blank"
						>
							View transaction on block explorer.
						</a>
					</p>
				{/if}
			</div>
		{/if}
	</Modal>
{/if}
