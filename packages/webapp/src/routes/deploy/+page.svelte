<script lang="ts">
	import { ValidStrategiesSection, InvalidStrategiesSection } from '@rainlanguage/ui-components';
	import type { PageData } from './$types';

	export let data: PageData;
	const { error, validStrategies, invalidStrategies } = data;
</script>

<div class="flex w-full max-w-6xl flex-col gap-y-6">
	<div class="text-4xl font-semibold text-gray-900 dark:text-white">Strategies</div>
	{#if error}
		<div class="flex gap-2 text-lg">
			Error loading registry:<span class="text-red-500">{error}</span>
		</div>
	{:else}
		<div class="bg-primary-100 dark:bg-primary-900 flex flex-col rounded-3xl p-12">
			<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
				Raindex empowers you to take full control of your trading strategies. All the strategies
				here are non-custodial, perpetual, and automated strategies built with our open-source,
				DeFi-native language <a class="underline" target="_blank" href="https://rainlang.xyz"
					>Rainlang</a
				>
			</h1>
		</div>
		{#if validStrategies.length === 0 && invalidStrategies.length === 0}
			<div class="text-center text-lg">No strategies found</div>
		{:else}
			{#if validStrategies.length > 0}
				<ValidStrategiesSection strategies={validStrategies} />
			{/if}
			{#if invalidStrategies.length > 0}
				<InvalidStrategiesSection strategiesWithErrors={invalidStrategies} />
			{/if}
		{/if}
	</div>
</div>
