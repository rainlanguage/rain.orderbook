<script lang="ts">
	import { PageHeader, StrategiesSection, InputRegistryUrl } from '@rainlanguage/ui-components';
	import { Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';

	const { error, strategyDetails } = $page.data;

	let advancedMode = localStorage.getItem('registry') ? true : false;
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname}>
	<svelte:fragment slot="actions">
		<Toggle checked={advancedMode} on:change={() => (advancedMode = !advancedMode)}>
			<span class="whitespace-nowrap">Advanced mode</span>
		</Toggle></svelte:fragment
	>
</PageHeader>

<div class="flex items-start justify-end gap-4">
	{#if advancedMode}
		<div class="mb-12 flex w-2/3 flex-col items-start gap-4">
			<InputRegistryUrl />
		</div>
	{/if}
</div>
<div class="flex w-full max-w-6xl flex-col gap-y-6">
	<div class="text-4xl font-semibold text-gray-900 dark:text-white">Strategies</div>

	{#if error}
		<div class="flex gap-2 text-lg">
			Error loading registry:<span class="text-red-500">{error}</span>
		</div>
	{:else}
		<div class="flex flex-col rounded-3xl bg-primary-100 p-12 dark:bg-primary-900">
			<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
				Raindex empowers you to take full control of your trading strategies. All the strategies
				here are non-custodial, perpetual, and automated strategies built with our open-source,
				DeFi-native language <a class="underline" target="_blank" href="https://rainlang.xyz"
					>Rainlang</a
				>
			</h1>
		</div>

		<StrategiesSection {strategyDetails} />
	{/if}
</div>
