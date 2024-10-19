<script lang="ts">
  import ButtonTab from '$lib/components/ButtonTab.svelte';
  import { nowTimestamp, TIME_DELTA_24_HOURS, TIME_DELTA_48_HOURS } from '$lib/services/time';
  import { ButtonGroup } from 'flowbite-svelte';

  let now = nowTimestamp();
  let timeDelta: number | undefined;

  function setNow() {
    now = nowTimestamp();
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
