<script lang="ts">
  import { P, TabItem, Tabs } from 'flowbite-svelte';
  import WordTable from '$lib/components/WordTable.svelte';
  import type { ScenarioAuthoringMeta } from '$lib/typeshare/authoringMeta';

  export let authoringMetas: ScenarioAuthoringMeta[] | undefined;
  export let error: unknown | undefined;
</script>

{#if authoringMetas}
  <Tabs
    style="underline"
    contentClass="mt-4"
    defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
  >
    {#each authoringMetas as scenario}
      <TabItem title={scenario.scenario_name}>
        <div class="flex gap-x-2 text-sm">
          {#if scenario.result.type === 'Success'}
            {#if scenario.result.data.deployer.result.type === 'Success'}
              <WordTable
                authoringMeta={scenario.result.data.deployer.result.data}
                pragma={scenario.result.data.deployer.address}
              />
            {:else if scenario.result.data.deployer.result.type === 'Error'}
              <div
                class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"
              >
                <p>Error getting words for this deployer:</p>
                <p>{scenario.result.data.deployer.result.data}</p>
              </div>
            {/if}
            {#each scenario.result.data.pragmas as pragma}
              {#if pragma.result.type === 'Success'}
                <WordTable authoringMeta={pragma.result.data} pragma={pragma.address} />
              {:else if pragma.result.type === 'Error'}
                <div
                  class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"
                >
                  <p>Error getting words for the pragma {pragma.address}:</p>
                  <p>{pragma.result.data}</p>
                </div>
              {/if}
            {/each}
          {:else if scenario.result.type === 'Error'}
            <div
              class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"
            >
              <p>Error getting words for this scenario:</p>
              <p>{scenario.result.data}</p>
            </div>
          {/if}
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
