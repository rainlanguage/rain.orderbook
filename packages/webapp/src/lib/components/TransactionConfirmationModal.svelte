<script lang="ts">
	import { Modal, Spinner, Button } from 'flowbite-svelte';
	import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
	import { match, P } from 'ts-pattern';
	import {
		handleWalletConfirmation,
		type WalletConfirmationState
	} from '../services/handleWalletConfirmation';

	export let open: boolean = false;
	export let modalTitle: string;
	export let closeOnConfirm: boolean = false;
	export let args: TransactionConfirmationProps['args'];

	let confirmationState: WalletConfirmationState = { status: 'awaiting_confirmation' };

	async function init() {
		const result = await handleWalletConfirmation(args);
		confirmationState = result.state;
		if (closeOnConfirm && confirmationState.status === 'confirmed') {
			open = false;
		}
	}

	$: if (open && args) {
		init();
	}

	$: if (!open) {
		confirmationState = { status: 'awaiting_confirmation' };
	}
</script>

<Modal size="sm" class="bg-opacity-90 backdrop-blur-sm" {open} data-testid="transaction-modal">
	{@const ui = match(confirmationState)
		.with({ status: 'awaiting_confirmation' }, () => ({
			iconType: 'spinner',
			title: modalTitle,
			description: 'Please confirm this transaction in your wallet.',
			showDismiss: true
		}))
		.with({ status: 'confirmed' }, () => ({
			iconType: 'success',
			title: 'Transaction submitted',
			description: 'Transaction has been submitted to the network.',
			showDismiss: true
		}))
		.with({ status: 'rejected' }, (state) => ({
			iconType: 'error',
			title: 'Confirmation failed',
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
					<Spinner color="blue" aria-label="Pending confirmation" size={10} />
				{:else if ui.iconType === 'success'}
					<h1 class="text-lg md:text-2xl" aria-label="Success">✅</h1>
				{:else if ui.iconType === 'error'}
					<h1 class="text-lg md:text-2xl" aria-label="Error">❌</h1>
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
				<Button on:click={() => (open = false)} aria-label="Close transaction modal">Dismiss</Button
				>
			{/if}
		</div>
	{/if}
</Modal>
