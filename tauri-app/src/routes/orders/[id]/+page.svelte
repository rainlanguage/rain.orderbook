<script lang="ts">
  import { Button, Card } from 'flowbite-svelte';
  import ArrowLeftSolid from 'flowbite-svelte-icons/ArrowLeftSolid.svelte';
  import { orderDetail } from '$lib/stores/orderDetail';
  import dayjs from 'dayjs';
  import utc from 'dayjs/plugin/utc';
  import bigIntSupport from 'dayjs/plugin/bigIntSupport';
  import { walletAddress } from '$lib/stores/settings';
  import ButtonLoading from '$lib/ButtonLoading.svelte';
  import ModalOrderRemove from '$lib/ModalOrderRemove.svelte';
  dayjs.extend(utc);
  dayjs.extend(bigIntSupport);

  export let data: { id: string };
  let showRemoveModal = false;

  orderDetail.refetch(data.id);
  $: order = $orderDetail[data.id];
</script>

<div class="flex w-full">
  <div class="flex-1">
    <Button outline size="xs" class="w-32" color="primary" href="/orders">
      <ArrowLeftSolid size="xs" /><span class="ml-2">All Orders</span>
    </Button>
  </div>
  <h1 class="flex-0 mb-8 text-4xl font-bold text-gray-900 dark:text-white">Order</h1>
  <div class="flex-1"></div>
</div>
{#if order === undefined}
  <div class="text-center text-gray-900 dark:text-white">Order not found</div>
{:else}
  <div class="flex w-full flex-wrap justify-evenly space-y-12 xl:space-x-8 2xl:space-y-0">
    <Card class="space-y-8" size="lg">
      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Order ID
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.id}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Owner Address
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.owner.id}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Created At
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {dayjs(BigInt(order.timestamp) * BigInt('1000'))
            .utc(true)
            .local()
            .format('DD/MM/YYYY h:mm A')}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Input Token(s)
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.valid_inputs?.map((t) => t.token_vault.token.name)}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Output Token(s)
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {order.valid_outputs?.map((t) => t.token_vault.token.name)}
        </p>
      </div>

      {#if $walletAddress !== '' && order.owner.id === $walletAddress}
        <div class="pt-4">
          <div class="flex justify-center space-x-20">
            <ButtonLoading color="blue" size="xl" on:click={() => (showRemoveModal = true)}>
              Remove
            </ButtonLoading>
          </div>
        </div>
      {/if}
    </Card>
  </div>
{/if}

<ModalOrderRemove bind:open={showRemoveModal} {order}/>