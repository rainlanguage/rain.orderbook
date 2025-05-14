<script lang="ts">
	import { getContext } from 'svelte';
	import type { TransactionManager } from '../../providers/transactions/TransactionManager';
	import type { RemoveOrder } from '../../models/RemoveOrderTransaction';
	import { TransactionStatusMessage } from '$lib/types/transaction';

	const transactionManager = getContext<TransactionManager>(
		'rain:ui-components:transactionManager'
	);

	const transactions = transactionManager.getTransactions();
</script>

<h1>Transactions</h1>

{#if $transactions.length === 0}
	<p>No transactions yet.</p>
{:else}
	<ul>
		{#each $transactions as transaction}
			<li>
				<p>Status: {transaction.state.status}</p>
				<p>Message: {transaction.state.message}</p>
				<p>Explorer Link: {transaction.state.explorerLink}</p>
			</li>
		{/each}
	</ul>
{/if}
