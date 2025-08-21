<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { OrdersListTable, PageHeader } from '@rainlanguage/ui-components';
</script>

<PageHeader title={'Orders'} pathname={$page.url.pathname} />

<OrdersListTable />
