<script lang="ts">
	import { PageHeader, StrategyShortTile } from '@rainlanguage/ui-components';
	import { Button, Input, Toggle } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';

	const { strategyDetails, error } = $page.data;

	let newRegistryUrl = '';
	let advancedMode = false;

	type StrategyDetail = {
		details: NameAndDescriptionCfg;
		name: string;
		dotrain: string;
		error?: unknown;
	};

	const validStrategies = strategyDetails.filter((strategy: StrategyDetail) => !strategy.error);
	const invalidStrategies = strategyDetails.filter((strategy: StrategyDetail) => strategy.error);

	const loadRegistryUrl = () => {
		// add the registry url to the url params
		window.history.pushState({}, '', window.location.pathname + '?registry=' + newRegistryUrl);
		// reload the page
		window.location.reload();
	};
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname}>
	<svelte:fragment slot="actions">
		<Toggle on:change={() => (advancedMode = !advancedMode)}>
			<span class="whitespace-nowrap">Advanced mode</span>
		</Toggle></svelte:fragment
	>
</PageHeader>

<div class="flex items-start justify-end gap-4">
	{#if advancedMode}
		<div class="mb-12 flex w-2/3 flex-col items-start gap-4">
			<div class="flex w-full items-start gap-4">
				<Input
					id="strategy-url"
					type="url"
					placeholder="Enter URL to raw strategy registry file"
					bind:value={newRegistryUrl}
				/>
				<Button class="text-nowrap" on:click={loadRegistryUrl}>Load Registry URL</Button>
			</div>
		</div>
	{/if}
</div>
<div class="flex w-full max-w-6xl flex-col gap-y-6">
	<div class="text-4xl font-semibold text-gray-900 dark:text-white">Strategies</div>

	{#if error}
		<div class="flex gap-2 text-xl">
			Error loading registry:<span class="text-red-500">{error}</span>
		</div>
	{:else}
		<div class="bg-primary-100 dark:bg-primary-900 flex flex-col rounded-3xl p-12">
			<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
				Raindex empowers you to take full control of your trading strategies. All the strategies
				here are non-custodial, perpetual, and automated strategies built with our open-source,
				DeFi-native language <a class="underline" target="_blank" href="https://rainlang.xyz"
					>Rainlang</a
				>.
			</h1>
		</div>

		{#if invalidStrategies.length > 0}
			<div class="flex flex-col gap-4 rounded-xl bg-red-100 p-6 dark:bg-red-900">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
					Invalid Strategies in registry
				</h2>
				<div class="flex flex-col gap-2">
					{#each invalidStrategies as strategy}
						<div class="flex flex-col gap-1">
							<span class="font-medium">{strategy.name}</span>
							<span class="text-red-600 dark:text-red-400">{strategy.error}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		{#if validStrategies.length > 0}
			{#key validStrategies}
				<div class="mb-36 grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
					{#each validStrategies as strategyDetail}
						<StrategyShortTile
							strategyDetails={strategyDetail.details}
							registryName={strategyDetail.name}
						/>
					{/each}
				</div>
			{/key}
		{/if}
	{/if}
</div>
