<script lang="ts">
	import { page } from '$app/stores';
	import { OrderPage } from '@rainlanguage/ui-components';
	const { dotrain, orderName } = $page.data;
</script>

<OrderPage {dotrain} {orderName} />
