<script lang="ts">
	import { page } from '$app/stores';
	import ToastListener from '$lib/components/ToastListener.svelte';
	const { orderHash } = $page.params;
</script>

<ToastListener queryKey={orderHash}>
	<slot />
</ToastListener>
