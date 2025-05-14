<script lang="ts">
	import type { ConfigSource } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import DropdownCheckbox from './DropdownCheckbox.svelte';

	export let settings: ConfigSource | undefined;
	export let activeSubgraphs: AppStoresInterface['activeSubgraphs'];

	$: dropdownOptions = Object.keys(settings?.subgraphs ?? {}).reduce(
		(acc, key) => ({
			...acc,
			[key]: key
		}),
		{}
	);

	function handleStatusChange(event: CustomEvent<Record<string, string>>) {
		let items = Object.keys(event.detail);
		activeSubgraphs.set(
			Object.values(items).reduce(
				(acc, key) => ({ ...acc, [key]: (settings?.subgraphs ?? {})[key] }),
				{} as Record<string, string>
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
