<script lang="ts">
	import { page } from '$app/stores';
	import { OrderTradesChart } from '@rainlanguage/ui-components';
	const { id, network } = $page.params;
	const { subgraphUrl, settings } = $page.data.stores;
</script>

<h1>Order Trades</h1>
<OrderTradesChart {id} {subgraphUrl} />
