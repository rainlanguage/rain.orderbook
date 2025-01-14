<script lang="ts">
	import {
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,
		SidebarBrand
	} from 'flowbite-svelte';
	import WalletConnect from './WalletConnect.svelte';
	import WalletSolid from 'flowbite-svelte-icons/WalletSolid.svelte';
	import ReceiptSolid from 'flowbite-svelte-icons/ReceiptSolid.svelte';
	import GearSolid from 'flowbite-svelte-icons/GearSolid.svelte';
	import FileLinesSolid from 'flowbite-svelte-icons/FileLinesSolid.svelte';
	import { IconExternalLink, IconTelegram } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import ButtonDarkMode from './ButtonDarkMode.svelte';
	import { PlusSolid } from 'flowbite-svelte-icons';

	export let colorTheme;
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-64 fixed z-10">
	<SidebarWrapper divClass="overflow-y-auto py-11 px-3 bg-gray-100 dark:bg-gray-800 min-h-screen">
		<SidebarGroup ulClass="">
			<div class="block dark:hidden">
				<SidebarBrand
					site={{
						name: '',
						href: '/',
						img: '/logo-light.svg'
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
						img: '/logo-dark.svg'
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
			<WalletConnect />
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
	</SidebarWrapper>
</Sidebar>
