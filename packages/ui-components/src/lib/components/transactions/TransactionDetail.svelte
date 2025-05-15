<script lang="ts">
	import type { TransactionState } from '$lib/models/Transaction';
	import { TransactionStatusMessage } from '$lib/types/transaction';
	import { type Writable } from 'svelte/store';
	import { match } from 'ts-pattern';
	export let state: Writable<TransactionState>;

	function getStatusEmoji(status: TransactionStatusMessage): string {
		return match(status)
			.with(TransactionStatusMessage.IDLE, () => '⏳')
			.with(TransactionStatusMessage.PENDING_REMOVE_ORDER, () => '🔄')
			.with(TransactionStatusMessage.PENDING_SUBGRAPH, () => '📊')
			.with(TransactionStatusMessage.SUCCESS, () => '✅')
			.with(TransactionStatusMessage.ERROR, () => '❌')
			.otherwise(() => '❓');
	}
</script>

<div class="w-full max-w-md rounded-md p-2 shadow-sm">
	<p class="mb-2 break-words p-1">{getStatusEmoji($state.status)} {$state.message}</p>
	<p class="break-words p-1">
		<a
			href={$state.explorerLink}
			target="_blank"
			rel="noopener noreferrer"
			class="text-blue-500 hover:underline">View transaction on explorer</a
		>
	</p>
</div>
