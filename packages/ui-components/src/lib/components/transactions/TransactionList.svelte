<script lang="ts">
	import { getContext } from 'svelte';
	import type { TransactionManager } from '../../providers/transactions/TransactionManager';
	import TransactionDetail from './TransactionDetail.svelte';

	const transactionManager = getContext<TransactionManager>(
		'rain:ui-components:transactionManager'
	);

	const transactionsStore = transactionManager.getTransactions();
</script>

<h1>Transactions</h1>

{#if $transactionsStore.length === 0}
	<p>No transactions yet.</p>
{:else}
	<ul>
		{#each $transactionsStore as transaction (transaction.state)}
			{@const state = transaction.state}
			<li>
				<TransactionDetail {state} />
			</li>
		{/each}
	</ul>
{/if}
