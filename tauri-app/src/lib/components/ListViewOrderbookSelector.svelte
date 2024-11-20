<script lang="ts">
  import { isEmpty } from 'lodash';
  import { DropdownOrderListAccounts } from '@rainlanguage/ui-components';
  import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
  import CheckboxZeroBalanceVault from './checkbox/CheckboxZeroBalanceVault.svelte';
  import { accounts, activeAccountsItems, activeSubgraphs, settings } from '$lib/stores/settings';
  import { Alert } from 'flowbite-svelte';
  import { page } from '$app/stores';
  import InputOrderHash from './input/InputOrderHash.svelte';
  import { DropdownActiveSubgraphs } from '@rainlanguage/ui-components';

  $: currentRoute = $page.url.pathname;
  $: isVaultsPage = currentRoute.startsWith('/vaults');
  $: isOrdersPage = currentRoute.startsWith('/orders');
</script>

<div class="flex min-w-[600px] items-center justify-end gap-x-2">
  {#if isEmpty($settings?.networks)}
    <Alert color="gray">
      No networks added to <a class="underline" href="/settings">settings</a>
    </Alert>
  {:else}
    {#if isVaultsPage}
      <CheckboxZeroBalanceVault />
    {/if}

    {#if isOrdersPage}
      <InputOrderHash />
      <DropdownOrderStatus />
    {/if}
    <DropdownOrderListAccounts {accounts} {activeAccountsItems} />
    <DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
  {/if}
</div>
