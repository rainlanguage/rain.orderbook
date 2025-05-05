<script lang="ts">
  import {
    Input,
    Modal,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';

  export let open = false;
  export let networks: Record<string, string> | undefined = undefined;
  export let blockNumbers: Record<number, number | undefined> = {};
</script>

<Modal
  class="z-50000"
  title="Set Debug Block Height For Networks"
  bind:open
  outsideclose
  size="sm"
  on:close={() => (open = false)}
>
  {#if networks}
    <Table divClass="rounded-lg overflow-hidden dark:border-none border overflow-x-scroll">
      <TableHead>
        <TableHeadCell>Network</TableHeadCell>
        <TableHeadCell>Block Height</TableHeadCell>
      </TableHead>

      <TableBody>
        {#each Object.entries(networks ?? {}).sort( (a, b) => (Number(a[0]) > Number(b[0]) ? 1 : -1), ) as [chainId, networkName]}
          <TableBodyRow>
            <TableBodyCell data-testid={`network-name-${chainId}`}>{networkName}</TableBodyCell>
            <TableBodyCell>
              <Input
                type="number"
                size="sm"
                class="self-center"
                placeHolder="Enter Block Height"
                value={blockNumbers[Number(chainId)]}
                on:input={(e) => {
                  if (e.currentTarget instanceof HTMLInputElement) {
                    blockNumbers[Number(chainId)] =
                      parseInt(e.currentTarget.value, 10) || undefined;
                  }
                }}
                data-testid={`chain-block-${chainId}`}
              ></Input>
            </TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {:else}
    Found no deployment
  {/if}
</Modal>
