<script lang="ts">
  import { P, TabItem, Tabs } from 'flowbite-svelte';
  import WordTable from '$lib/components/WordTable.svelte';
  import type { ScenarioWords } from '@rainlanguage/orderbook/js_api';

  export let authoringMetas: ScenarioWords[] | undefined;
  export let error: unknown | undefined;
</script>

{#if authoringMetas}
  <Tabs
    style="underline"
    contentClass="mt-4"
    defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
  >
    {#each authoringMetas as scenario}
      <TabItem title={scenario.scenario}>
        <div class="flex gap-x-2 text-sm">
          {#if scenario.deployerWords.words.type === 'Success'}
            <WordTable
              authoringMeta={scenario.deployerWords.words.data}
              pragma={scenario.deployerWords.address}
            />
          {:else if scenario.deployerWords.words.type === 'Error'}
            <div
              data-testid="deployer-error-msg"
              class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"
            >
              <p>Error getting words for this deployer:</p>
              <p>{scenario.deployerWords.words.data}</p>
            </div>
          {/if}
          {#each scenario.pragmaWords as pragma}
            {#if pragma.words.type === 'Success'}
              <WordTable authoringMeta={pragma.words.data} pragma={pragma.address} />
            {:else if pragma.words.type === 'Error'}
              <div
                class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"
                data-testid="pragma-error-msg"
              >
                <p>Error getting words for the pragma {pragma.address}:</p>
                <p>{pragma.words.data}</p>
              </div>
            {/if}
          {/each}
        </div>
      </TabItem>
    {/each}
  </Tabs>
{:else if error}
  <div data-testid="error-msg">
    <P>Error getting words for this order</P>
    <P>{error?.toString() || ''}</P>
  </div>
{/if}
