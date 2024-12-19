<script lang="ts">
	import ButtonTab from '../ButtonTab.svelte';
	import { dateTimestamp, TIME_DELTA_1_YEAR, TIME_DELTA_30_DAYS } from '$lib/services/time';
	import { ButtonGroup } from 'flowbite-svelte';

	let now = dateTimestamp(new Date());
	let timeDelta: number | undefined;

	function setNow() {
		now = dateTimestamp(new Date());
	}

	export let startTimestamp: number | undefined;
	export let endTimestamp: number | undefined;
</script>

<ButtonGroup class="bg-gray-800" data-testid="lightweightChartYearButtons">
	<ButtonTab
		on:click={() => {
			setNow();
			timeDelta = undefined;
			startTimestamp = undefined;
			endTimestamp = undefined;
		}}
		active={timeDelta === undefined}
		size="xs"
		class="px-2 py-1">All Time</ButtonTab
	>
	<ButtonTab
		on:click={() => {
			setNow();
			timeDelta = TIME_DELTA_1_YEAR;
			startTimestamp = now - TIME_DELTA_1_YEAR;
			endTimestamp = now;
		}}
		active={timeDelta === TIME_DELTA_1_YEAR}
		size="xs"
		class="px-2 py-1">Last Year</ButtonTab
	>
	<ButtonTab
		on:click={() => {
			setNow();
			timeDelta = TIME_DELTA_30_DAYS;
			startTimestamp = now - TIME_DELTA_30_DAYS;
			endTimestamp = now;
		}}
		active={timeDelta === TIME_DELTA_30_DAYS}
		size="xs"
		class="px-2 py-1">Last Month</ButtonTab
	>
</ButtonGroup>
