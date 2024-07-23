<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import {
    activeOrderbook,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
  } from '$lib/stores/settings';
  import { onMount } from 'svelte';
  import VaultListTable from '$lib/components/tables/VaultListTable.svelte';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Vaults" />

<VaultListTable />
