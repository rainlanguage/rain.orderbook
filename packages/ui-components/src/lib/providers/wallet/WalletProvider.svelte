<script lang="ts">
	import { readable } from 'svelte/store';
	import { setAccountContext } from './context';
	import type { Account } from '$lib/types/account';

	export let account: Account = readable(null);

	setAccountContext(account);
</script>

<slot />
