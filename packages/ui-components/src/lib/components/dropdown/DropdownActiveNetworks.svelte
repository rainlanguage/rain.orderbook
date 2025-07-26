<script lang="ts">
	import type { AppStoresInterface } from '$lib/types/appStores';
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	const raindexClient = useRaindexClient();

	export let selectedChainIds: AppStoresInterface['selectedChainIds'];

	let dropdownOptions: Record<string, string> = {};
	$: {
		const uniqueChainIds = raindexClient.getUniqueChainIds();
		if (uniqueChainIds.error) {
			dropdownOptions = {};
		} else {
			dropdownOptions = Object.fromEntries(
				uniqueChainIds.value.map((chainId) => [
					String(chainId),
					getNetworkName(chainId) ?? `Chain ${chainId}`
				])
			);
		}
	}

	function handleStatusChange(event: CustomEvent<Record<string, string>>) {
		const chainIds = Object.keys(event.detail).map(Number);
		selectedChainIds.set(chainIds);
	}

	let value: Record<string, string> = {};
	$: {
		const networks = raindexClient.getAllNetworks();
		if (networks.error) {
			value = {};
		} else {
			value = Object.fromEntries(
				$selectedChainIds.map((chainId) => [
					String(chainId),
					getNetworkName(chainId) ?? `Chain ${chainId}`
				])
			);
		}
	}
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
