<script lang="ts">
	import type { NetworkCfg, NewConfig } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import DropdownCheckbox from './DropdownCheckbox.svelte';

	export let settings: NewConfig;
	export let activeNetworks: AppStoresInterface['activeNetworks'];

	$: dropdownOptions = Object.keys(settings.orderbook.networks ?? {}).reduce(
		(acc, key) => ({
			...acc,
			[key]: key
		}),
		{}
	);

	function handleStatusChange(event: CustomEvent<Record<string, string>>) {
		let items = Object.keys(event.detail);
		activeNetworks.set(
			Object.values(items).reduce(
				(acc, key) => ({ ...acc, [key]: (settings.orderbook.networks ?? {})[key] }),
				{} as Record<string, NetworkCfg>
			)
		);
	}

	$: value =
		Object.keys($activeNetworks).length === 0
			? {}
			: Object.keys($activeNetworks).reduce(
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
