<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { Config, SubgraphCfg } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores';

	export let settings: Config;
	export let activeSubgraphs: AppStoresInterface['activeSubgraphs'];

	$: dropdownOptions = Object.keys(settings.orderbook.subgraphs ?? {}).reduce(
		(acc, key) => ({
			...acc,
			[key]: key
		}),
		{}
	);

	function handleStatusChange(event: CustomEvent<Record<string, SubgraphCfg>>) {
		let items = Object.keys(event.detail);
		activeSubgraphs.set(
			Object.values(items).reduce(
				(acc, key) => ({ ...acc, [key]: (settings.orderbook.subgraphs ?? {})[key] }),
				{} as Record<string, SubgraphCfg>
			)
		);
	}

	$: value =
		Object.keys($activeSubgraphs).length === 0
			? {}
			: Object.keys($activeSubgraphs).reduce(
					(acc, key) => ({
						...acc,
						[key]: key
					}),
					{}
				);
</script>

<div data-testid="subgraphs-dropdown">
	<DropdownCheckbox
		options={dropdownOptions}
		on:change={handleStatusChange}
		label="Networks"
		showAllLabel={false}
		onlyTitle={true}
		{value}
	/>
</div>
