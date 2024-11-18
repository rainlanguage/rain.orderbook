<script lang="ts">
	import { Hash, HashType } from '@rainlanguage/ui-components';
	import OrdersListTable from '@rainlanguage/ui-components';
</script>

<Hash value="0x1234567890abcdef" type={HashType.Wallet} shorten={true} />
<OrdersListTable connectedWalletAddress={''} />
