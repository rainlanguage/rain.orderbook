<script lang="ts">
	import { Toast } from 'flowbite-svelte';
	import { slide } from 'svelte/transition';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import type { ToastProps } from '$lib/types/toast';
	import { useToasts } from '$lib/providers/toasts/useToasts';

	export let toast: ToastProps;
	export let i: number;

	const { removeToast } = useToasts();
</script>

<Toast
	on:close={() => removeToast(i)}
	dismissable={true}
	transition={slide}
	color={toast.color}
	class="mb-2"
>
	<svelte:fragment slot="icon">
		{#if toast.type === 'success'}
			<CheckCircleSolid class="h-5 w-5" data-testid="success-icon" />
		{:else if toast.type === 'error'}
			<CloseCircleSolid class="h-5 w-5" data-testid="error-icon" />
		{/if}
	</svelte:fragment>
	<p class="font-semibold">{toast.message}</p>
	{#if toast.detail}
		<p>{toast.detail}</p>
	{/if}
	{#if toast.links}
		<div class="flex flex-col">
			{#each toast.links as { link, label }}
				<a
					href={link}
					target="_blank"
					rel="noopener noreferrer"
					class="text-blue-500 hover:underline"
				>
					{label}
				</a>
			{/each}
		</div>
	{/if}
</Toast>
