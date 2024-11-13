<script lang="ts">
	import { OrdersListTable } from '@rainlanguage/ui-components';
</script>

<OrdersListTable connectedWalletAddress={'0xf08bCbce72f62c95Dcb7c07dCb5Ed26ACfCfBc11'}>
	<svelte:fragment slot="filters"
		><div class="border-4 border-red-500 p-2 text-xl">Place for filters</div></svelte:fragment
	>
</OrdersListTable>
