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
  export let networks: Record<number, string> | undefined = undefined;
  export let blockNumbers: Record<number, number | undefined> = {};
  export let onClose: () => void;
</script>

<Modal
  title="Debug Block Height"
  bind:open
  outsideclose
  size="sm"
  on:close={() => {
    onClose();
    open = false;
  }}
  backdropClass="fixed inset-0 z-40 bg-gray-900 bg-opacity-50 dark:bg-opacity-80 z-[1000] backdrop-class-id"
  dialogClass="fixed top-0 start-0 end-0 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex z-[1000]"
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
                placeholder="Enter Block Height"
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
    Found no deployment, please add deployments to your order's configurations to debug it
  {/if}
</Modal>
