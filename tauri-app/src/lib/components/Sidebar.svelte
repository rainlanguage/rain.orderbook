<script lang="ts">
  import {
    Sidebar,
    SidebarGroup,
    SidebarItem,
    SidebarWrapper,
    SidebarBrand,
  } from 'flowbite-svelte';
  import HomeSolid from 'flowbite-svelte-icons/HomeSolid.svelte';
  import WalletSolid from 'flowbite-svelte-icons/WalletSolid.svelte';
  import ReceiptSolid from 'flowbite-svelte-icons/ReceiptSolid.svelte';
  import GearSolid from 'flowbite-svelte-icons/GearSolid.svelte';
  import IconExternalLink from '$lib/components/IconExternalLink.svelte';
  import { page } from '$app/stores';
  import { allRequiredSettingsValid } from '$lib/stores/settings';
  import ButtonDarkMode from './ButtonDarkMode.svelte';

  $: nonActiveClass = $allRequiredSettingsValid
    ? 'flex items-center p-2 text-base font-normal text-gray-900 rounded-lg dark:text-white hover:bg-gray-200 dark:hover:bg-gray-600'
    : 'flex items-center p-2 text-base font-normal text-gray-900 rounded-lg dark:text-white/25 ';
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-64 fixed">
  <SidebarWrapper divClass="overflow-y-auto py-11 px-3 bg-gray-100 dark:bg-gray-800 min-h-screen">
    <SidebarGroup>
      <SidebarBrand
        site={{
          name: 'Rain Orderbook',
          href: '/',
          img: '/logo.svg',
        }}
        imgClass="w-8 ml-2"
        aClass="w-full flex items-center justify-start gap-x-3 mb-5"
        spanClass="block text-xl font-regular dark:text-white"
      ></SidebarBrand>
    </SidebarGroup>
    <SidebarGroup border>
      <SidebarItem label="Dashboard" href="/" {nonActiveClass}>
        <svelte:fragment slot="icon">
          <HomeSolid class="h-5 w-5" />
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Vaults" href="/vaults" {nonActiveClass}>
        <svelte:fragment slot="icon">
          <WalletSolid class="h-5 w-5" />
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Orders" href="/orders" {nonActiveClass}>
        <svelte:fragment slot="icon">
          <ReceiptSolid class="h-5 w-5" />
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border>
      <SidebarItem label="Settings" href="/settings">
        <svelte:fragment slot="icon">
          <GearSolid class="h-5 w-5" />
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Documentation" target="_blank" href="https://docs.rainlang.xyz/intro">
        <svelte:fragment slot="icon">
          <IconExternalLink />
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border class="flex justify-end">
      <ButtonDarkMode />
    </SidebarGroup>
  </SidebarWrapper>
</Sidebar>
