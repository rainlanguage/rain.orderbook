<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import { Button } from 'flowbite-svelte';
  import { ordersList } from '$lib/stores/ordersList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import TableOrders from '$lib/components/TableOrders.svelte';

  redirectIfSettingsNotDefined();
</script>

<PageHeader title="Orders"></PageHeader>

<div class="flex w-full justify-between py-4">
  <div class="text-3xl font-medium dark:text-white">Orders</div>
  <Button color="green" href="/orders/add">Add Order</Button>
</div>

<TableOrders {ordersList} />
