<script lang="ts">
  import { Modal } from 'flowbite-svelte';
  import EvalResultsTable from '../debug/EvalResultsTable.svelte';
  import type { RainEvalResultsTable } from '@rainlanguage/orderbook';
  export let open: boolean;
  export let pair: string;
  export let data: RainEvalResultsTable;
</script>

<Modal title={`Debugging scenario for pair ${pair}`} bind:open outsideclose size="lg">
  <div class="flex items-center">
    <div class="flex w-full items-center justify-end"></div>
  </div>
  <EvalResultsTable table={data} />
  <div class="flex flex-col gap-y-2 text-sm"></div>
</Modal>
