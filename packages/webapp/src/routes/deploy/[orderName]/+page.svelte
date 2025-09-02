<script lang="ts">
	import { page } from '$app/stores';
	import { OrderPage } from '@rainlanguage/ui-components';
	const { orderName } = $page.data;
</script>

<OrderPage {orderName} />
