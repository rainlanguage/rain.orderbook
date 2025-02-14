<script lang="ts">
	import { StrategySection, PageHeader } from '@rainlanguage/ui-components';
	import { Button, Input, Spinner, Toggle, Textarea } from 'flowbite-svelte';
	import { registryUrl } from '$lib/stores/registry';
	import { getFileRegistry } from './getFileRegistry';
	import { onMount } from 'svelte';
	import { rawDotrain } from '$lib/stores/raw-dotrain';
	import { page } from '$app/stores';
	import { getTransactionAddOrders } from '@rainlanguage/orderbook/js_api';

	let files: { name: string; url: string }[] = [];
	let inputDotrain = '';
	let error = '';
	let errorDetails = '';
	let loading = false;
	let advancedMode = false;

	onMount(() => {
		fetchFilesFromRegistry($registryUrl);
	});

	const fetchFilesFromRegistry = async (url: string) => {
		loading = true;
		try {
			files = await getFileRegistry(url);
		} catch (e) {
			error = 'Error getting registry';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
		loading = false;
	};

	const loadRawStrategy = () => {
		if (inputDotrain.trim()) {
			files = [];
			$rawDotrain = inputDotrain;
			inputDotrain = '';
		}
	};

	const loadRegistryUrl = () => {
		fetchFilesFromRegistry($registryUrl);
		// add the registry url to the url params
		window.history.pushState({}, '', window.location.pathname + '?registry=' + $registryUrl);
	};
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname}>
	<svelte:fragment slot="actions">
		<Toggle on:change={() => (advancedMode = !advancedMode)}>
			{'Advanced Mode'}
		</Toggle></svelte:fragment
	>
</PageHeader>
<div class="container flex w-full flex-col">
	<div class="flex items-start justify-end gap-4">
		{#if advancedMode}
			<div class="mb-12 flex w-2/3 flex-col items-start gap-4">
				<div class="flex w-full items-start gap-4">
					<Input
						id="strategy-url"
						type="url"
						placeholder="Enter URL to raw strategy registry file"
						bind:value={$registryUrl}
					/>
					<Button class="text-nowrap" on:click={loadRegistryUrl}>Load Registry URL</Button>
				</div>
				<div class="flex w-full items-start gap-4">
					<Textarea
						id="textarea-id"
						placeholder="Raw strategy"
						rows="8"
						bind:value={inputDotrain}
					/>
					<Button class="text-nowrap" on:click={loadRawStrategy}>Load Raw Strategy</Button>
				</div>
			</div>
		{/if}
	</div>

	<div
		class="bg-primary-100 dark:bg-primary-900 mb-14 mt-8 flex max-w-6xl flex-col rounded-3xl p-12"
	>
		<div class="flex flex-col gap-y-4">
			<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
				Raindex empowers you to take full control of your trading strategies. All the strategies
				here are non-custodial, perpetual, and automated strategies built with our open-source,
				DeFi-native language <a class="underline" target="_blank" href="https://rainlang.xyz"
					>Rainlang</a
				>.
			</h1>
			<p class="text-base text-gray-600 dark:text-gray-400"></p>
		</div>
		<div class="flex flex-col gap-y-4">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white">How to deploy</h2>
			<ol class="list-outside list-decimal space-y-2 pl-5">
				<li class="text-base text-gray-600 dark:text-gray-200">
					Choose the strategy that aligns with your convictions (and always DYOR).
				</li>
				<li class="text-base text-gray-600 dark:text-gray-200">
					Follow the instructions to configure and deploy the strategy. If you need help,
					<a class="underline" target="_blank" href="https://t.me/+W0aQ36ptN_E2MjZk"
						>join the Telegram group</a
					>
				</li>
				<li class="text-base text-gray-600 dark:text-gray-200">
					Monitor the performance of the strategy, deposit and withdraw funds at any time.
				</li>
			</ol>
		</div>
	</div>

	{#if loading}
		<Spinner />
	{:else if error}
		<p>{error}</p>
		<p>{errorDetails}</p>
	{/if}
	{#if files.length > 0}
		{#key files}
			<div class="mb-36 flex flex-col gap-y-24">
				{#each files as { name, url }}
					<StrategySection strategyUrl={url} strategyName={name} />
				{/each}
			</div>
		{/key}
	{:else if $rawDotrain}
		<StrategySection rawDotrain={$rawDotrain} strategyName={'raw'} />
	{/if}
</div>
