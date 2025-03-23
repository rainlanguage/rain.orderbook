<script lang="ts">
	import type { Writable } from 'svelte/store';
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { ConfigSource } from '@rainlanguage/orderbook';

	export let settings: ConfigSource;
	export let activeSubgraphs: Writable<Record<string, string>>;

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
