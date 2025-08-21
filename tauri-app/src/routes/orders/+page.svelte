<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { OrdersListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
</script>

<PageHeader title="Orders" pathname={$page.url.pathname} />

<OrdersListTable />
