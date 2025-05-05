<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import ToastDetail from '$lib/components/ToastDetail.svelte';
	import { setToastsContext } from './context';
	import { writable, type Writable } from 'svelte/store';
	import type { ToastProps } from '$lib/types/toast';
	import { fade } from 'svelte/transition';
	export let toasts: Writable<ToastProps[]> = writable([]);

	setToastsContext(toasts);
</script>

<div class="fixed right-4 top-4 z-[100]">
	{#each $toasts as toast}
		<div out:fade>
			<ToastDetail {toast} />
		</div>
	{/each}
</div>

<slot />
