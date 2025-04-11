<script lang="ts">
	import {
		useRegistry,
		fetchRegistryDotrains,
		validateStrategies
	} from '@rainlanguage/ui-components';

	const registry = useRegistry();

	const getStrategies = async () => {
		const registryDotrains = await fetchRegistryDotrains($registry.getCurrentRegistry());
		const strategies = await validateStrategies(registryDotrains);
		return strategies;
	};

	const customRegistryParam = $registry.isCustomRegistry()
		? `?registry=${$registry.getCurrentRegistry()}`
		: '';
</script>

<div class="flex w-full max-w-6xl flex-col gap-y-6">
	<div class="text-4xl font-semibold text-gray-900 dark:text-white">Strategies</div>

	{#await getStrategies() then { validStrategies, invalidStrategies }}
		<div class="flex flex-col rounded-3xl bg-primary-100 p-12 dark:bg-primary-900">
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
				<div
					class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3"
					data-testid="valid-strategies"
				>
					{#each validStrategies as strategy}
						<a
							href={`/deploy/${strategy.name}${customRegistryParam}`}
							data-testid="strategy-short-tile"
							class="flex flex-col gap-y-2 rounded-xl border border-gray-200 p-4 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-900"
						>
							<h3 class="text-2xl font-semibold text-gray-900 dark:text-white">
								{strategy.details.name}
							</h3>
							<p class="text-lg text-gray-600 dark:text-gray-400">
								{strategy.details.short_description}
							</p>
						</a>
					{/each}
				</div>
			{/if}
			{#if invalidStrategies.length > 0}
				<div
					class="flex flex-col gap-4 rounded-xl bg-red-100 p-6 dark:bg-red-900"
					data-testid="invalid-strategies"
				>
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
		{/if}
	{:catch error}
		<div class="flex gap-2 text-lg">
			Failed to load strategies:<span class="text-red-500">{error}</span>
		</div>
	{/await}
</div>
