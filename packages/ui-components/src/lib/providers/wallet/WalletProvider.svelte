<script context="module" lang="ts">
	export const ACCOUNT_KEY = 'account_key';
</script>

<script lang="ts">
	import { readable, type Readable } from 'svelte/store';
	import { setAccountContext } from './context';
	import type { Account } from '$lib/types/account';

	export let account: Account = readable(null);

	setAccountContext(account);
</script>

<slot />
