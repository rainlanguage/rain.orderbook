<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import VaultDetail from '$lib/components/detail/VaultDetail.svelte';
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail id={$page.params.id} />
