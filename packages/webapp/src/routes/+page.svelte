<script>
	import { Hash, HashType } from '@rainlanguage/ui-components';
</script>

<div data-testid="page-container" class="p-24">
	<Hash value="0x1234567890abcdef" type={HashType.Address} shorten={true} copyOnClick={true} />
</div>
