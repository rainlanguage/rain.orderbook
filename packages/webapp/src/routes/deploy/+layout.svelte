<script lang="ts">
	import CustomRegistryWarning from '$lib/components/CustomRegistryWarning.svelte';
    import { InputRegistryUrl, PageHeader, useRegistry } from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import { slide } from 'svelte/transition';
    const { isCustomRegistry } = useRegistry();
    let showAdvanced = false;
    $: isDeployPage = $page.url.pathname === '/deploy';
</script>

<PageHeader title={$page.data.pageName || 'Deploy'} pathname={$page.url.pathname} />
    <div class="flex flex-col gap-2">
		<div class="flex w-full content-end items-end justify-between">
				{#if $isCustomRegistry}
					<CustomRegistryWarning />
				{:else if isDeployPage}
					<div class="ml-auto"></div>
				{/if}
				{#if isDeployPage}
					<Toggle checked={showAdvanced} on:change={() => (showAdvanced = !showAdvanced)}>
						<span class="whitespace-nowrap">Advanced mode</span>
					</Toggle>
				{/if}
			</div>
			<div class="flex flex-col items-end gap-4">
				{#if showAdvanced && isDeployPage}
					<div in:slide class="w-full">
						<InputRegistryUrl />
					</div>
				{/if}
			</div>
		</div>
    <slot></slot>
