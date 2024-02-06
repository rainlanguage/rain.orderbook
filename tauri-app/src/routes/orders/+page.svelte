<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Button,
  } from 'flowbite-svelte';
  import { ordersList } from '$lib/stores/ordersList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import TableOrders from '$lib/components/TableOrders.svelte';

  redirectIfSettingsNotDefined();
  ordersList.fetchFirst();
</script>

<PageHeader title="Orders">
  <svelte:fragment slot="actions">
    <Button color="green" size="xs">Add</Button>
  </svelte:fragment>
</PageHeader>

<TableOrders {ordersList} />
