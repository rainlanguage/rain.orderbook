<script lang="ts">
	import type { AppStoresInterface } from '$lib/types/appStores';
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	const raindexClient = useRaindexClient();

	export let selectedChainIds: AppStoresInterface['selectedChainIds'];

	let dropdownOptions: Record<string, string> = {};
	let validChainIds: number[] = [];
	$: {
		const uniqueChainIds = raindexClient.getUniqueChainIds();
		if (uniqueChainIds.error) {
			dropdownOptions = {};
			validChainIds = [];
		} else {
			validChainIds = uniqueChainIds.value;
			dropdownOptions = Object.fromEntries(
				validChainIds.map((chainId) => [
					String(chainId),
					getNetworkName(chainId) ?? `Chain ${chainId}`
				])
			);
		}
	}

	$: {
		const filtered = $selectedChainIds.filter((chainId) => validChainIds.includes(chainId));
		if (filtered.length !== $selectedChainIds.length) {
			selectedChainIds.set(filtered);
		}
	}

	function handleStatusChange(event: CustomEvent<Record<string, string>>) {
		const chainIds = Object.keys(event.detail)
			.map(Number)
			.filter((chainId) => validChainIds.includes(chainId));
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
