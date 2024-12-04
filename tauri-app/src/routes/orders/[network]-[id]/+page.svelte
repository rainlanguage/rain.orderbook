<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import OrderDetail from '$lib/components/detail/OrderDetail.svelte';
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

<OrderDetail id={$page.params.id} network="{$page.params.network}"/>
