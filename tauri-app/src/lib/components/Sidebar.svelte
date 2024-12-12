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
  import FileLinesSolid from 'flowbite-svelte-icons/FileLinesSolid.svelte';
  import IconExternalLink from '$lib/components/IconExternalLink.svelte';
  import { page } from '$app/stores';
  import ButtonDarkMode from '../../../../packages/ui-components/src/lib/components/ButtonDarkMode.svelte';
  import { PlusSolid } from 'flowbite-svelte-icons';
  import IconTelegram from '$lib/components/IconTelegram.svelte';
  import ModalConnect from '$lib/components/ModalConnect.svelte';
  import { onMount } from 'svelte';
  import { getAppCommitSha } from '$lib/services/app';
  import { colorTheme } from '$lib/stores/darkMode';

  let app_sha: string;
  onMount(async () => {
    app_sha = await getAppCommitSha();
  });
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-64 fixed z-10">
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
      <SidebarItem label="New Order" href={'/orders/add'}>
        <svelte:fragment slot="icon">
          <PlusSolid class="h-5 w-5" />
          <span data-testid="sidebar-new-order"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Orders" href="/orders">
        <svelte:fragment slot="icon">
          <ReceiptSolid class="h-5 w-5" />
          <span data-testid="sidebar-orders"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem label="Vaults" href="/vaults">
        <svelte:fragment slot="icon">
          <WalletSolid class="h-5 w-5" />
          <span data-testid="sidebar-vaults"></span>
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border>
      <ModalConnect />
    </SidebarGroup>
    <SidebarGroup border>
      <SidebarItem label="Settings" href="/settings">
        <svelte:fragment slot="icon">
          <GearSolid class="h-5 w-5" />
          <span data-testid="sidebar-settings"></span>
        </svelte:fragment>
      </SidebarItem>
      <SidebarItem
        label="Documentation"
        target="_blank"
        href="https://docs.rainlang.xyz/raindex/overview"
      >
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
      <SidebarItem label="License" href="/license">
        <svelte:fragment slot="icon">
          <FileLinesSolid />
          <span data-testid="sidebar-license"></span>
        </svelte:fragment>
      </SidebarItem>
    </SidebarGroup>
    <SidebarGroup border class="flex justify-start">
      <ButtonDarkMode {colorTheme} />
    </SidebarGroup>
    <SidebarGroup border class="flex justify-start self-end">
      <div class="flex flex-col text-xs text-gray-500 dark:text-gray-400">
        <p>Raindex version commit:</p>
        <p class="break-all">
          {app_sha}
        </p>
      </div>
    </SidebarGroup>
  </SidebarWrapper>
</Sidebar>
