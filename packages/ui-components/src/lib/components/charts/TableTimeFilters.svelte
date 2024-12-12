<script lang="ts">
	import ButtonTab from '../ButtonTab.svelte';
	import { ButtonGroup } from 'flowbite-svelte';

	const TIME_DELTA_24_HOURS = 60 * 60 * 24;
	const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;

	let now = Math.floor(new Date().getTime() / 1000);
	let timeDelta: number | undefined;

	function setNow() {
		now = Math.floor(new Date().getTime() / 1000);
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
			timeDelta = TIME_DELTA_48_HOURS;
			startTimestamp = now - TIME_DELTA_48_HOURS;
			endTimestamp = now;
		}}
		active={timeDelta === TIME_DELTA_48_HOURS}
		size="xs"
		class="px-2 py-1">48 Hours</ButtonTab
	>
	<ButtonTab
		on:click={() => {
			setNow();
			timeDelta = TIME_DELTA_24_HOURS;
			startTimestamp = now - TIME_DELTA_24_HOURS;
			endTimestamp = now;
		}}
		active={timeDelta === TIME_DELTA_24_HOURS}
		size="xs"
		class="px-2 py-1">24 Hours</ButtonTab
	>
</ButtonGroup>
