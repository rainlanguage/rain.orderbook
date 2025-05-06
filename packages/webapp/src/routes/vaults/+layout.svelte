<script lang="ts">
	import ToastListener from '$lib/components/ToastListener.svelte';
	import { page } from '$app/stores';
	const { id } = $page.params;
</script>

<ToastListener queryKey={id}>
	<slot />
</ToastListener>
