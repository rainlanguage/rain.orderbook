<script lang="ts">
	import type { TransactionState } from '$lib/models/Transaction';
	import { TransactionStatusMessage } from '$lib/types/transaction';
	import { type Writable } from 'svelte/store';
	import { match } from 'ts-pattern';
	export let state: Writable<TransactionState>;

	function getStatusEmoji(status: TransactionStatusMessage): string {
		return match(status)
			.with(TransactionStatusMessage.IDLE, () => '⏳')
			.with(TransactionStatusMessage.PENDING_RECEIPT, () => '🔄')
			.with(TransactionStatusMessage.PENDING_SUBGRAPH, () => '📊')
			.with(TransactionStatusMessage.SUCCESS, () => '✅')
			.with(TransactionStatusMessage.ERROR, () => '❌')
			.otherwise(() => '❓');
	}
</script>

<div
	class="flex w-full flex-col gap-1 rounded-md border border-gray-200 p-1 shadow-sm dark:border-gray-800"
>
	<p class="break-words font-semibold">{$state.name}</p>
	<p class="break-words">{getStatusEmoji($state.status)} {$state.status}</p>
	{#if $state.errorDetails}
		<p class="break-words">{$state.errorDetails}</p>
	{/if}
	{#if $state.explorerLink}
		<p class="break-words">
			<a
				href={$state.explorerLink}
				target="_blank"
				rel="noopener noreferrer"
				class="text-blue-500 hover:underline">View transaction on explorer</a
			>
		</p>
	{/if}
</div>
