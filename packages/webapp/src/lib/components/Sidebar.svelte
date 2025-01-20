<script lang="ts">
	import {
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,
		SidebarBrand
	} from 'flowbite-svelte';
	import WalletSolid from 'flowbite-svelte-icons/WalletSolid.svelte';
	import ReceiptSolid from 'flowbite-svelte-icons/ReceiptSolid.svelte';
	import FileLinesSolid from 'flowbite-svelte-icons/FileLinesSolid.svelte';
	import { page } from '$app/stores';
	import {
		ButtonDarkMode,
		IconTelegram,
		IconExternalLink,
		logoDark,
		logoLight
	} from '@rainlanguage/ui-components';
	import { PlusOutline } from 'flowbite-svelte-icons';
	export let colorTheme;
</script>

<Sidebar activeUrl={$page.url.pathname} asideClass="w-64 fixed z-10">
	<SidebarWrapper divClass="overflow-y-auto py-11 px-3 bg-gray-100 dark:bg-gray-800 min-h-screen">
		<SidebarGroup ulClass="">
			<SidebarBrand
				site={{
					name: '',
					href: '/',
					img: colorTheme == 'light' ? logoDark : logoLight
				}}
				imgClass="m-auto"
			></SidebarBrand>
		</SidebarGroup>
		<SidebarGroup border>
			<SidebarItem label="Deploy" href="/deploy">
				<svelte:fragment slot="icon">
					<PlusOutline class="h-5 w-5" />
					<span data-testid="sidebar-deploy"></span>
				</svelte:fragment>
			</SidebarItem>
		</SidebarGroup>
		<SidebarGroup border>
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
