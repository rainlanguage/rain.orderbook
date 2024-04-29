<script lang="ts">
  import type { ScenariosAuthoringMeta } from '$lib/typeshare/dotrainOrder';
  import { TabItem, Tabs } from 'flowbite-svelte';
  import WordTable from '$lib/components/WordTable.svelte';

  export let authoringMetas: ScenariosAuthoringMeta;
</script>

{#if authoringMetas}
  <Tabs
    style="underline"
    contentClass="mt-4"
    defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
  >
    {#each Object.entries(authoringMetas) as [scenario, pragmas]}
      <TabItem title={scenario}>
        <div class="flex gap-x-2 text-sm">
          {#each Object.entries(pragmas) as [pragma, authoringMeta]}
            <div>
              <WordTable {authoringMeta} {pragma} />
            </div>
          {/each}
        </div>
      </TabItem>
    {/each}
  </Tabs>
{/if}
