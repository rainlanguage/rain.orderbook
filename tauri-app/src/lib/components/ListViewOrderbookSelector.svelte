<script lang="ts">
  import { isEmpty } from 'lodash';
  import CheckboxZeroBalanceVault from './checkbox/CheckboxZeroBalanceVault.svelte';
  import {
    accounts,
    activeAccountsItems,
    activeSubgraphs,
    settings,
    activeOrderStatus,
    orderHash,
  } from '$lib/stores/settings';
  import { Alert } from 'flowbite-svelte';
  import { page } from '$app/stores';
  import {
    DropdownActiveSubgraphs,
    DropdownOrderStatus,
    DropdownOrderListAccounts,
    InputOrderHash,
  } from '@rainlanguage/ui-components';

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
      <InputOrderHash {orderHash} />
      <DropdownOrderStatus {activeOrderStatus} />
    {/if}
    <DropdownOrderListAccounts {accounts} {activeAccountsItems} />
    <DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
  {/if}
</div>
