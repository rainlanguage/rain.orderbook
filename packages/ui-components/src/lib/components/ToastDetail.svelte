<script lang="ts">
	import { Toast } from 'flowbite-svelte';
	import { slide } from 'svelte/transition';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import type { ToastProps } from '$lib/types/toast';
	export let toast: ToastProps;
</script>

<Toast dismissable={true} transition={slide} color={toast.color} class="mb-2">
	<svelte:fragment slot="icon">
		{#if toast.type === 'success'}
			<CheckCircleSolid class="h-5 w-5" />
		{:else if toast.type === 'error'}
			<CloseCircleSolid class="h-5 w-5" />
		{/if}
	</svelte:fragment>
	{toast.message}
</Toast>
