<script lang="ts">
  import {
    Sidebar,
    SidebarGroup,
    SidebarItem,
    SidebarWrapper,
    SidebarBrand,
    DarkMode,
  } from 'flowbite-svelte';
  import HomeSolid from 'flowbite-svelte-icons/HomeSolid.svelte';
  import WalletSolid from 'flowbite-svelte-icons/WalletSolid.svelte';
  import ReceiptSolid from 'flowbite-svelte-icons/ReceiptSolid.svelte';
  import LifeBuoySolid from 'flowbite-svelte-icons/LifeBuoySolid.svelte';
  import GearSolid from 'flowbite-svelte-icons/GearSolid.svelte';
  import IconExternalLink from './IconExternalLink.svelte';
  import { page } from '$app/stores';
  import { isSettingsDefinedAndValid } from './stores/settings';

  $: nonActiveClass = $isSettingsDefinedAndValid
    ? 'flex items-center p-2 text-base font-normal text-green-900 rounded-lg dark:text-white hover:bg-green-100 dark:hover:bg-green-700'
    : 'flex items-center p-2 text-base font-normal text-green-900 rounded-lg dark:text-white/25';
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-52 fixed left-2 top-2">
  <SidebarWrapper divClass="overflow-y-auto py-4 px-3 rounded bg-gray-100 dark:bg-gray-800">
    <SidebarGroup>
      <SidebarBrand
        site={{
          name: 'Rain Orderbook',
          href: '/',
          img: '/logo.svg',
        }}
        imgClass="w-8"
        aClass="w-full flex items-center justify-between space-x-4 mb-5"
        spanClass="block text-xl font-bold dark:text-white"
      ></SidebarBrand>
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
      <SidebarItem label="Help" href="/help">
        <svelte:fragment slot="icon">
          <LifeBuoySolid class="h-5 w-5" />
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Documentation" target="_blank" href="https://docs.rainlang.xyz/intro">
        <svelte:fragment slot="icon">
          <IconExternalLink />
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border class="flex justify-end">
      <DarkMode />
    </SidebarGroup>
  </SidebarWrapper>
</Sidebar>
