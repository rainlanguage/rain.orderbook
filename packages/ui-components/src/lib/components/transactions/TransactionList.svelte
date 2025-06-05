<script lang="ts">
	import TransactionDetail from './TransactionDetail.svelte';
	import { useTransactions } from '$lib/providers/transactions/useTransactions';
	const { transactions } = useTransactions();
</script>

{#if $transactions.length > 0}
	<ul aria-label="Transaction list" class="flex w-full flex-col gap-4">
		{#each $transactions.slice().reverse() as transaction}
			{@const state = transaction.state}
			<li>
				<TransactionDetail {state} />
			</li>
		{/each}
	</ul>
{/if}
