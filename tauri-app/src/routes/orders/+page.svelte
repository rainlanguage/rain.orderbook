<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import {
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
    activeOrderbook,
  } from '$lib/stores/settings';
  import { onMount } from 'svelte';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Orders" />
