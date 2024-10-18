<script lang="ts">
  import ButtonTab from '$lib/components/ButtonTab.svelte';
  import { ButtonGroup } from 'flowbite-svelte';

  const TIME_DELTA_1_MONTH = 60 * 60 * 24 * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_1_MONTH * 12;

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
      timeDelta = TIME_DELTA_1_YEAR;
      startTimestamp = now - TIME_DELTA_1_YEAR;
      endTimestamp = now;
    }}
    active={timeDelta === TIME_DELTA_1_YEAR}
    size="xs"
    class="px-2 py-1">1 Year</ButtonTab
  >
  <ButtonTab
    on:click={() => {
      setNow();
      timeDelta = TIME_DELTA_1_MONTH;
      startTimestamp = now - TIME_DELTA_1_MONTH;
      endTimestamp = now;
    }}
    active={timeDelta === TIME_DELTA_1_MONTH}
    size="xs"
    class="px-2 py-1">1 Month</ButtonTab
  >
</ButtonGroup>
