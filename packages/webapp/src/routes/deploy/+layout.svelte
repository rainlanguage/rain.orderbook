<script lang="ts">
	import { PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import TransactionModal from '$lib/components/TransactionModal.svelte';
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname} />
<slot />
<TransactionModal />
