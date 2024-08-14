<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import OrderDetail from '$lib/components/detail/OrderDetail.svelte';
</script>

<PageHeader title="Order" />

<OrderDetail id={$page.params.id} />
