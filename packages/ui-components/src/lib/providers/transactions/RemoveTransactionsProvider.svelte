<script lang="ts">
	import { writable, type Writable } from 'svelte/store';
	import { setTransactionsContext } from './context';
	import type { BaseTransaction } from '$lib/types/transaction';

	export let transactions: Writable<BaseTransaction[]> = writable([]);

	setTransactionsContext(transactions);
</script>

<slot />
