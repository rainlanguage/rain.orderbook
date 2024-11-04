<script lang="ts">
  import { Checkbox, Label } from 'flowbite-svelte';
  import { hideZeroBalanceVaults } from '$lib/stores/settings';

  function handleHideZeroBalanceChange() {
    $hideZeroBalanceVaults = !$hideZeroBalanceVaults;
  }
</script>

<div class="flex items-center gap-x-2">
  <Label
    for="hide-empty-vaults"
    class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
  >
    Hide empty vaults
  </Label>
  <Checkbox
    id="hide-empty-vaults"
    checked={$hideZeroBalanceVaults}
    on:change={handleHideZeroBalanceChange}
  />
</div>
