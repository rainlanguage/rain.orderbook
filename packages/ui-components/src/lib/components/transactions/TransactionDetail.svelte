<script lang="ts">
	import type { TransactionStoreState } from '$lib/models/Transaction';
	import { TransactionStatusMessage } from '$lib/types/transaction';
	import { type Readable } from 'svelte/store';
	import { match } from 'ts-pattern';

	export let state: Readable<TransactionStoreState>;

	function getStatusEmoji(status: TransactionStatusMessage): string {
		return match(status)
			.with(TransactionStatusMessage.PENDING_RECEIPT, () => '🔄')
			.with(TransactionStatusMessage.PENDING_SUBGRAPH, () => '📊')
			.with(TransactionStatusMessage.SUCCESS, () => '✅')
			.with(TransactionStatusMessage.ERROR, () => '❌')
			.otherwise(() => '❓');
	}
</script>

<div class="flex w-full flex-col gap-1 rounded-md bg-gray-300 p-2 dark:bg-gray-700">
	<p class="break-words font-semibold">{$state.name}</p>
	<div class="flex flex-col gap-1 text-sm">
		<p class="break-words">{getStatusEmoji($state.status)} {$state.status}</p>
		{#if $state.errorDetails}
			<p class="break-words">{$state.errorDetails}</p>
		{/if}
		{#if $state.links.length > 0}
			<div class="flex flex-col gap-1">
				{#each $state.links as link}
					<p class="break-words">
						<a
							href={link.link}
							data-testid={`transaction-detail-link-${link.label}`}
							target="_blank"
							rel="noopener noreferrer"
							class="text-blue-500 hover:underline"
						>
							{link.label}
						</a>
					</p>
				{/each}
			</div>
		{/if}
	</div>
</div>
