<script>
	import { Hash, HashType, OrdersListTable } from '@rainlanguage/ui-components';
</script>

<div data-testid="page-container" class="p-24">
	<Hash value="0x1234567890abcdef" type={HashType.Address} shorten={true} copyOnClick={true} />
	<OrdersListTable connectedWalletAddress="0xf08bCbce72f62c95Dcb7c07dCb5Ed26ACfCfBc11">
		<svelte:fragment slot="filters">
			<div class="border-4 border-red-500 p-2 text-xl">Place for filters</div>
		</svelte:fragment>
	</OrdersListTable>
</div>
