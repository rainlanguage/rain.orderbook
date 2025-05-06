<script lang="ts">
	import ToastListener from '$lib/components/TransactionsListener.svelte';
	import { page } from '$app/stores';
	const { id } = $page.params;
</script>

<ToastListener queryKey={id}>
	<slot />
</ToastListener>
