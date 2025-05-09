<script lang="ts">
	import TransactionsListener from '$lib/components/TransactionsListener.svelte';
	import { page } from '$app/stores';
	const { id } = $page.params;
</script>

<TransactionsListener queryKey={id}>
	<slot />
</TransactionsListener>
