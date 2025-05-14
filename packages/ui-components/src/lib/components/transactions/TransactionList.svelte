<script lang="ts">
	import { getContext } from 'svelte';
	import type { TransactionManager } from '../../providers/transactions/TransactionManager';
	import TransactionDetail from './TransactionDetail.svelte';
	import type { RemoveOrderTransaction } from '../../models/RemoveOrderTransaction';

	const transactionManager = getContext<TransactionManager>(
		'rain:ui-components:transactionManager'
	);

	const transactionsStore = transactionManager.getTransactions();
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
