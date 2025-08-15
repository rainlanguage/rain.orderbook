<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable />
