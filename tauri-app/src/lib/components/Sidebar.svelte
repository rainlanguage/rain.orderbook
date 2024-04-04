<script lang="ts">
  import {
    Sidebar,
    SidebarGroup,
    SidebarItem,
    SidebarWrapper,
    SidebarBrand,
  } from 'flowbite-svelte';
  import WalletSolid from 'flowbite-svelte-icons/WalletSolid.svelte';
  import ReceiptSolid from 'flowbite-svelte-icons/ReceiptSolid.svelte';
  import GearSolid from 'flowbite-svelte-icons/GearSolid.svelte';
  import IconExternalLink from '$lib/components/IconExternalLink.svelte';
  import { page } from '$app/stores';
  import ButtonDarkMode from '$lib/components/ButtonDarkMode.svelte';
  import DropdownActiveNetwork from '$lib/components/DropdownActiveNetwork.svelte';
  import DropdownActiveOrderbook from '$lib/components/DropdownActiveOrderbook.svelte';
  import IconTelegram from '$lib/components/IconTelegram.svelte';

  export let hasRequiredSettings = false;

  $: nonActiveClass = !hasRequiredSettings
    ? 'flex items-center p-2 text-base font-normal text-gray-900 rounded-lg dark:text-white/25 '
    : 'flex items-center p-2 text-base font-normal text-gray-900 rounded-lg dark:text-white hover:bg-gray-200 dark:hover:bg-gray-600';
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-64 fixed">
  <SidebarWrapper divClass="overflow-y-auto py-11 px-3 bg-gray-100 dark:bg-gray-800 min-h-screen">
    <SidebarGroup ulClass="">
      <div class="block dark:hidden">
        <SidebarBrand
          site={{
            name: '',
            href: '/',
            img: '/logo-light.svg',
          }}
          imgClass="w-2/3 m-auto"
          aClass="w-full flex items-center justify-start gap-x-3 mb-5"
          spanClass="hidden"
        ></SidebarBrand>
      </div>
      <div class="hidden dark:block">
        <SidebarBrand
          site={{
            name: '',
            href: '/',
            img: '/logo-dark.svg',
          }}
          imgClass="w-2/3 m-auto"
          aClass="w-full flex items-center justify-start gap-x-3 mb-5"
          spanClass="hidden"
        ></SidebarBrand>
      </div>
    </SidebarGroup>
    <SidebarGroup border>
      <SidebarItem
        label="Orders"
        href={hasRequiredSettings ? '/orders' : undefined}
        {nonActiveClass}
      >
        <svelte:fragment slot="icon">
          <ReceiptSolid class="h-5 w-5" />
          <span data-testid="sidebar-orders"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem
        label="Vaults"
        href={hasRequiredSettings ? '/vaults' : undefined}
        {nonActiveClass}
      >
        <svelte:fragment slot="icon">
          <WalletSolid class="h-5 w-5" />
          <span data-testid="sidebar-vaults"></span>
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border>
      <DropdownActiveNetwork />
      <DropdownActiveOrderbook />
    </SidebarGroup>
    <SidebarGroup border>
      <SidebarItem label="Settings" href="/settings">
        <svelte:fragment slot="icon">
          <GearSolid class="h-5 w-5" />
          <span data-testid="sidebar-settings"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Documentation" target="_blank" href="https://docs.rainlang.xyz/intro">
        <svelte:fragment slot="icon">
          <IconExternalLink />
          <span data-testid="sidebar-documentation"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Ask for help" target="_blank" href="https://t.me/+W0aQ36ptN_E2MjZk">
        <svelte:fragment slot="icon">
          <IconTelegram />
          <span data-testid="sidebar-telegram"></span>
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border class="flex justify-end">
      <ButtonDarkMode />
    </SidebarGroup>
  </SidebarWrapper>
</Sidebar>
