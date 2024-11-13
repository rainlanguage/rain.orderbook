<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import OrdersListTable from '$lib/components/tables/OrdersListTable.svelte';
  import {
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
    activeOrderbook,
  } from '$lib/stores/settings';
  import { onMount } from 'svelte';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Orders" />
<OrdersListTable />
