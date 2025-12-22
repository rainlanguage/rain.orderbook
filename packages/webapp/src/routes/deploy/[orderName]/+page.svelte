<script lang="ts">
	import { page } from '$app/stores';
	import { OrderPage } from '@rainlanguage/ui-components';
	const { orderName, orderDetail, deployments } = $page.data;
</script>

<OrderPage {orderName} {orderDetail} {deployments} />
