<script lang="ts">
  import { accounts, activeAccountsItems } from '$lib/stores/settings';
  import DropdownCheckbox from './DropdownCheckbox.svelte';

  $: options = $accounts;
</script>

<DropdownCheckbox
  {options}
  bind:value={$activeAccountsItems}
  label="Accounts"
  allLabel="All accounts"
  emptyMessage="No accounts added"
/>
