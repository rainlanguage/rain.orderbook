<script lang="ts">
  import { Radio, Badge, Button, Label } from 'flowbite-svelte';
  import { reportErrorToSentry } from "$lib/services/sentry";
  import { getAddressFromLedger } from "$lib/services/wallet";
  import { toasts } from "$lib/stores/toasts";
  import type { DerivationAddress } from "$lib/types/derivation";
  import { find, range } from "lodash";
  import SkeletonRow from '$lib/components/SkeletonRow.svelte';
    import { ChevronDownSolid } from 'flowbite-svelte-icons';

  export let address: string;
  export let index: number;
  let derivationsList: Array<DerivationAddress> = [];
  let isLoading: boolean;
  let open: boolean = false;

  $: {
    if(index !== undefined) {
      const val = find(derivationsList, (d) => d.index === index);
      if(val?.address) {
        address = val.address;
      }
    }
  };

  $: {
    if(!isLoading && open && derivationsList.length < 5) {
      fetchMore();
    }
  };

  $: address, open = false;

  async function fetchMore(count = 5) {
    if(isLoading) return;

    isLoading = true;
    try {
      const derivationIndexMin = derivationsList.length > 0 ? derivationsList.length : 0;
      for (let index of range(derivationIndexMin, derivationIndexMin+count)) {
        if(!open) break;

        const address = await getAddressFromLedger(index);
        derivationsList = [...derivationsList, {
          address,
          index
        }];
      }
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(`Ledger error: ${e as string}`);
    }
    isLoading = false;
  }
</script>

<div class="relative">
  <Button color="alternative" class="w-full pl-2 pr-0 text-left flex justify-between overflow-hidden overflow-ellipsis" on:click={() => (open = !open)}>
    <Label>Select by Derivation Index</Label>
    <ChevronDownSolid class="w-3 h-3 mx-2 text-black dark:text-white" />
  </Button>

  {#if open}
    <div class="absolute z-50 shadow-lg rounded-md bg-gray-100 p-4 w-full overflow-y-scroll">
      <div>
        {#each derivationsList as derivation}
          <Radio bind:group={index} value={derivation.index} class="w-full p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600">
            <div class="ml-2 flex justify-start items-center space-x-2">
              <Badge>{derivation.index}</Badge>
              <div class="w-72 font-normal text-gray-800 dark:text-white">{derivation.address}</div>
            </div>
          </Radio>
        {/each}
        {#if isLoading}
          <SkeletonRow />
        {:else}
          <div class="flex justify-end">
            <Button color="alternative" class="w-full" on:click={() => {fetchMore();}} size="xs">Load More</Button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>