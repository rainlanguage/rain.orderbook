<script lang="ts">
	import TransactionDetail from './TransactionDetail.svelte';
	import { useTransactions } from '../../providers/transactions/useTransactions';
	const { transactions } = useTransactions();
</script>

{#if $transactions.length > 0}
	<ul aria-label="Transaction list" class="transaction-list">
		{#each $transactions as transaction}
			{@const state = transaction.state}
			<li>
				<TransactionDetail {state} />
			</li>
		{/each}
	</ul>
{/if}
