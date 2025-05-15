<script lang="ts">
	import TransactionDetail from './TransactionDetail.svelte';
	import { useTransactions } from '../../providers/transactions/useTransactions';
	const { manager } = useTransactions();

	const transactionsStore = manager.getTransactions();
</script>

{#if $transactionsStore.length > 0}
	<ul>
		{#each $transactionsStore as transaction (transaction.state)}
			{@const state = transaction.state}
			<li>
				<TransactionDetail {state} />
			</li>
		{/each}
	</ul>
{/if}
