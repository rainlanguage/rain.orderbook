<script lang="ts">
	import { setContext } from 'svelte';
	import { SQLiteWasmDatabase } from '@rainlanguage/sqlite-web';
	import { LOCAL_DB_CONTEXT_KEY } from '../hooks/useLocalDb';

	export let localDb: SQLiteWasmDatabase;

	setContext(LOCAL_DB_CONTEXT_KEY, localDb);
</script>

<slot />
