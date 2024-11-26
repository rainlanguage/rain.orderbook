<script lang="ts">
	import ButtonTab from '../ButtonTab.svelte';
	import { ButtonGroup } from 'flowbite-svelte';

	const TIME_DELTA_24_HOURS = 60 * 60 * 24;
	const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
	const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
	const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

	export let timeDelta = TIME_DELTA_1_YEAR;
</script>

<ButtonGroup class="bg-gray-800" data-testid="lightweightChartYearButtons">
	<ButtonTab
		on:click={() => (timeDelta = TIME_DELTA_1_YEAR)}
		active={timeDelta === TIME_DELTA_1_YEAR}
		size="xs"
		class="px-2 py-1">1 Year</ButtonTab
	>
	<ButtonTab
		on:click={() => (timeDelta = TIME_DELTA_30_DAYS)}
		active={timeDelta === TIME_DELTA_30_DAYS}
		size="xs"
		class="px-2 py-1">30 Days</ButtonTab
	>
	<ButtonTab
		on:click={() => (timeDelta = TIME_DELTA_7_DAYS)}
		active={timeDelta === TIME_DELTA_7_DAYS}
		size="xs"
		class="px-2 py-1">7 Days</ButtonTab
	>
	<ButtonTab
		on:click={() => (timeDelta = TIME_DELTA_24_HOURS)}
		active={timeDelta === TIME_DELTA_24_HOURS}
		size="xs"
		class="px-2 py-1">24 Hours</ButtonTab
	>
</ButtonGroup>
