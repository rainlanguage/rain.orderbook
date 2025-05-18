<script lang="ts">
	import { page } from '$app/stores';
	import TransactionsListener from '$lib/components/TransactionsListener.svelte';
	const { orderHash } = $page.params;
</script>

<TransactionsListener queryKey={orderHash}>
	<slot />
</TransactionsListener>
