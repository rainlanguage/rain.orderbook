<script lang="ts">
  import { BreadcrumbItem, Card } from 'flowbite-svelte';
  import { orderDetail } from '$lib/stores/orderDetail';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import BadgeActive from '$lib/components/BadgeActive.svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import ButtonVaultLink from '$lib/components/ButtonVaultLink.svelte';
  import { orderRemove } from '$lib/utils/orderRemove';
  import PageHeader from '$lib/components/PageHeader.svelte';

  export let data: { id: string };
  let isSubmitting = false;

  $: order = $orderDetail[data.id];

  async function remove() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  orderDetail.refetch(data.id);
</script>

<PageHeader title="Order">
  <svelte:fragment slot="breadcrumbs">
    <BreadcrumbItem href="/orders">Orders</BreadcrumbItem>
  </svelte:fragment>
</PageHeader>

{#if order === undefined}
  <div class="text-center text-gray-900 dark:text-white">Order not found</div>
{:else}
  <div class="flex w-full flex-wrap justify-evenly space-y-12 xl:space-x-8 2xl:space-y-0">
    <Card class="relative" size="lg">
      <BadgeActive active={order.order_active} class="absolute right-5 top-5"/>
      <div class="mt-4">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Order ID
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.id}
        </p>
      </div>

      <div class="mt-8">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Owner Address
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.owner.id}
        </p>
      </div>

      <div class="mt-8">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Created At
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {formatTimestampSecondsAsLocal(BigInt(order.timestamp))}
        </p>
      </div>

      <div class="mt-8">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Input Vaults
        </h5>
        <div class="flex flex-wrap space-x-2 space-y-2">
          {#each (order.valid_inputs || []) as t}
            <ButtonVaultLink tokenVault={t.token_vault} />
          {/each}
        </div>
      </div>

      <div class="mt-8">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Output Vaults
        </h5>
        <div class="flex flex-wrap space-x-2 space-y-2">
          {#each (order.valid_outputs || []) as t}
            <ButtonVaultLink tokenVault={t.token_vault} />
          {/each}
        </div>
      </div>

      {#if $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
        <div class="mt-8">
          <div class="flex justify-center space-x-20">
            <ButtonLoading color="blue" size="xl" on:click={remove} loading={isSubmitting}>
              Remove
            </ButtonLoading>
          </div>
        </div>
      {/if}
    </Card>
  </div>
{/if}

