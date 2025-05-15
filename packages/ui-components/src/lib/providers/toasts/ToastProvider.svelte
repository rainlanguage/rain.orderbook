<script lang="ts">
	import ToastDetail from '$lib/components/ToastDetail.svelte';
	import { setToastsContext } from './context';
	import { writable, type Writable } from 'svelte/store';
	import type { ToastProps } from '$lib/types/toast';
	import { fade } from 'svelte/transition';
	const toasts: Writable<ToastProps[]> = writable([]);
	setToastsContext(toasts);
</script>

<div class="fixed right-4 top-4 z-[100]">
	{#each $toasts as toast}
		<div out:fade data-testid="toast">
			<ToastDetail {toast} />
		</div>
	{/each}
</div>

<slot />
