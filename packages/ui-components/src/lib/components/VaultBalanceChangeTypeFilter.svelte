<script lang="ts">
	import { type VaultBalanceChangeFilter } from '@rainlanguage/orderbook';
	import DropdownCheckbox from './dropdown/DropdownCheckbox.svelte';

	export let value: VaultBalanceChangeFilter[] | undefined = undefined;

	const filterOptions: Record<VaultBalanceChangeFilter, string> = {
		deposit: 'Deposit',
		withdrawal: 'Withdrawal',
		takeOrder: 'Take order',
		clear: 'Clear',
		clearBounty: 'Clear Bounty'
	};

	let typeFilter: Partial<Record<VaultBalanceChangeFilter, string>> = {};

	$: {
		const keys = Object.keys(typeFilter) as VaultBalanceChangeFilter[];
		if (keys.length > 0 && keys.length < Object.keys(filterOptions).length) {
			value = keys.sort();
		} else {
			value = undefined;
		}
	}
</script>

<DropdownCheckbox
	options={filterOptions}
	bind:value={typeFilter}
	label="Change Type"
	allLabel="All types"
	onlyTitle={true}
/>
