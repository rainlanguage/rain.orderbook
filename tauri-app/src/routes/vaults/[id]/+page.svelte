<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import VaultDetail from '$lib/components/detail/VaultDetail.svelte';
</script>

<PageHeader title="Vault" />

<VaultDetail id={$page.params.id} />
