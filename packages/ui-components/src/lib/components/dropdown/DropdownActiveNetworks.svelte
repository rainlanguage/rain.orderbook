<script lang="ts">
	import type { NewConfig } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import { getNetworkName } from '$lib/utils/getNetworkName';

	export let settings: NewConfig;
	export let selectedChainIds: AppStoresInterface['selectedChainIds'];

	$: dropdownOptions = Object.keys(settings.orderbook.networks ?? {}).reduce((acc, key) => {
		const networkCfg = (settings.orderbook.networks ?? {})[key];
		const networkName = getNetworkName(Number(networkCfg.chainId)) ?? key;

		// Check if we already have a network with this chain ID
		const existingKey = Object.keys(acc).find((existingKey) => {
			const existingNetworkCfg = (settings.orderbook.networks ?? {})[existingKey];
			return existingNetworkCfg.chainId === networkCfg.chainId;
		});
		if (!existingKey) {
			return {
				...acc,
				[key]: networkName
			};
		}
		return acc;
	}, {});

	function handleStatusChange(event: CustomEvent<Record<string, string>>) {
		let items = Object.keys(event.detail);
		const chainIds = Array.from(
			new Set(
				Object.values(items).map((key) => {
					const networkCfg = (settings.orderbook.networks ?? {})[key];
					return networkCfg.chainId;
				})
			)
		);
		selectedChainIds.set(chainIds);
	}

	$: value = $selectedChainIds.reduce(
		(acc, chainId) => {
			// Find the first network key that matches this chain ID
			const networkKey = Object.keys(settings.orderbook.networks ?? {}).find((key) => {
				const networkCfg = (settings.orderbook.networks ?? {})[key];
				return networkCfg.chainId === chainId;
			});
			if (networkKey) {
				const networkName = getNetworkName(chainId) ?? networkKey;
				acc[networkKey] = networkName;
			}
			return acc;
		},
		{} as Record<string, string>
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
