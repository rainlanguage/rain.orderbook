<script lang="ts">
	import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Spinner, Button } from 'flowbite-svelte';
	import { sendTransaction, switchChain } from '@wagmi/core';
	import type { Hex } from 'viem';
	import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
	import { match, P } from 'ts-pattern';

	export let open: boolean = false;
	export let args: TransactionConfirmationProps['args'];

	type WalletConfirmationState =
		| { status: 'awaiting_confirmation' }
		| { status: 'confirmed' }
		| { status: 'rejected'; reason: string }
		| { status: 'error'; reason: string };

	let transactionHash: Hex | undefined;

	let confirmationState: WalletConfirmationState = { status: 'awaiting_confirmation' };

	async function handleWalletConfirmation() {
		try {
			await switchChain($wagmiConfig, { chainId: args.chainId });
		} catch (error) {
			return (confirmationState = {
				status: 'error',
				reason: error instanceof Error ? error.message : 'Failed to switch chain'
			});
		}
		try {
			const calldata = await args.getCalldataFn();

			transactionHash = await sendTransaction($wagmiConfig, {
				to: args.orderbookAddress,
				data: calldata as Hex
			});

			confirmationState = { status: 'confirmed' };
			args.onConfirm(transactionHash);
		} catch {
			confirmationState = {
				status: 'rejected',
				reason: 'User rejected transaction'
			};
		}
	}

	handleWalletConfirmation();
</script>

<Modal size="sm" class="bg-opacity-90 backdrop-blur-sm" bind:open data-testid="transaction-modal">
	{@const ui = match(confirmationState)
		.with({ status: 'awaiting_confirmation' }, () => ({
			iconType: 'spinner',
			title: 'Waiting for wallet confirmation',
			description: 'Please confirm this transaction in your wallet.',
			showDismiss: true
		}))
		.with({ status: 'confirmed' }, () => ({
			iconType: 'success',
			title: 'Transaction Submitted',
			description: 'Transaction has been submitted to the network.',
			showDismiss: false
		}))
		.with({ status: 'rejected' }, (state) => ({
			iconType: 'error',
			title: 'Transaction rejected',
			description: state.reason,
			showDismiss: true
		}))
		.with({ status: 'error' }, (state) => ({
			iconType: 'error',
			title: 'Confirmation failed',
			description: state.reason,
			showDismiss: true
		}))
		.otherwise(() => null)}

	{#if ui}
		<div class="flex flex-col items-center justify-center gap-2 p-4">
			<div
				class={match(confirmationState)
					.with(
						{ status: P.union('rejected', 'error') },
						() =>
							'mb-4 flex h-16 w-16 items-center justify-center rounded-full border-2 border-red-400 bg-red-100 dark:bg-red-900'
					)
					.with(
						{ status: 'confirmed' },
						() =>
							'mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900'
					)
					.otherwise(
						() =>
							'mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-100 dark:bg-blue-900'
					)}
			>
				{#if ui.iconType === 'spinner'}
					<Spinner color="blue" size={10} />
				{:else if ui.iconType === 'success'}
					<h1 class="text-lg md:text-2xl">✅</h1>
				{:else if ui.iconType === 'error'}
					<h1 class="text-lg md:text-2xl">❌</h1>
				{/if}
			</div>

			<div class="flex flex-col gap-4 text-center">
				<div class="flex flex-col">
					<p
						class="w-full whitespace-pre-wrap break-words text-center text-lg font-semibold text-gray-900 dark:text-white"
					>
						{ui.title}
					</p>
					<p
						class="w-full whitespace-pre-wrap break-words text-center text-sm font-normal text-gray-900 dark:text-white"
					>
						{ui.description}
					</p>
				</div>
			</div>

			{#if ui.showDismiss}
				<Button on:click={() => (open = false)}>Dismiss</Button>
			{/if}
		</div>
	{/if}
</Modal>
