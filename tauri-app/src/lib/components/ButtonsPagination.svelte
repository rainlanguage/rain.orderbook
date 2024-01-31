<script lang="ts">
  import { ArrowLeftOutline, ArrowRightOutline } from "flowbite-svelte-icons";
	import { createEventDispatcher } from 'svelte';
  import { Button, Spinner } from "flowbite-svelte";

  const dispatch = createEventDispatcher();

  export let index: number = 1;
  export let loading = false;
</script>

<div class="flex items-center">
  {#if loading}
    <Spinner class="mr-4 h-5 w-5" color="white" />
  {/if}
  <Button class="rounded-none rounded-l-lg border-1" disabled={loading} on:click={() => dispatch('previous')} color="light">
    <ArrowLeftOutline class="w-3.5 h-3.5 me-2" />
    Previous
  </Button>
  <div class="text-center font-medium justify-center px-5 py-2.5 text-sm bg-white border border-gray-300 dark:bg-gray-800 dark:text-white dark:border-gray-600">
    {index}
  </div>
  <Button class="rounded-none rounded-r-lg border-1" disabled={loading} on:click={() => dispatch('next')} color="light">
    Next
    <ArrowRightOutline class="w-3.5 h-3.5 ms-2" />
  </Button>
</div>
