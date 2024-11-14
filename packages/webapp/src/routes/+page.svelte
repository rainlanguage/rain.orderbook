<script lang="ts">
	import { Hash, HashType } from '@rainlanguage/ui-components';
</script>

<div data-testid="page-container">
	<Hash value="0x1234567890abcdef" type={HashType.Wallet} shorten={true} />
</div>
