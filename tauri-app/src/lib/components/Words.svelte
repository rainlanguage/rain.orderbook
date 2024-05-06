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
              <P>Error getting words for this deployer:</P>
              <P>{scenario.result.data.deployer.result.data}</P>
            {/if}
            {#each scenario.result.data.pragmas as pragma}
              {#if pragma.result.type === 'Success'}
                <WordTable authoringMeta={pragma.result.data} pragma={pragma.address} />
              {:else if pragma.result.type === 'Error'}
                <P>Error getting words for this pragma:</P>
                <P>{pragma.result.data}</P>
              {/if}
            {/each}
          {:else if scenario.result.type === 'Error'}
            <P>Error getting words for this scenario:</P>
            <P>{scenario.result.data}</P>
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
