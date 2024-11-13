<script lang="ts">
	import { Hash } from '@rainlanguage/ui-components';

	enum HashType {
		Identifier,
		Wallet,
		Transaction,
		Address
	}
</script>

<Hash value="0x1234567890abcdef" type={HashType.Wallet} shorten={true} />
