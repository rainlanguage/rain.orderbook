<script>
	import {
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,
		SidebarBrand,
		DarkMode
	} from 'flowbite-svelte';
	import {
		HomeSolid,
		WalletSolid,
		ReceiptSolid,
		QuestionCircleSolid,
		LifeBuoySolid,
		OpenBookSolid
	} from 'flowbite-svelte-icons';
	import { page } from '$app/stores';
	$: activeUrl = $page.url.pathname;
	let activeClass =
		'flex items-center p-2 text-base font-normal text-primary-900 bg-primary-200 dark:bg-primary-700 rounded-lg dark:text-white hover:bg-primary-100 dark:hover:bg-gray-700';
	let nonActiveClass =
		'flex items-center p-2 text-base font-normal text-green-900 rounded-lg dark:text-white hover:bg-green-100 dark:hover:bg-green-700';
</script>

<Sidebar {activeUrl} {activeClass} {nonActiveClass} asideClass="w-64 " class="h-full">
	<SidebarWrapper divClass="overflow-y-auto py-4 px-3 rounded bg-gray-100 dark:bg-gray-800">
		<SidebarGroup>
			<SidebarBrand
				site={{
					name: 'Rain Orderbook',
					href: '/',
					img: '/logo.svg'
				}}
				imgClass="w-8"
				aClass="w-full flex items-center justify-between space-x-4 mb-5"
				spanClass="block text-xl font-bold dark:text-white"
			></SidebarBrand>
			<SidebarItem label="Dashboard" href="/">
				<svelte:fragment slot="icon">
					<HomeSolid class="h-5 w-5" />
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Vaults" href="/vaults">
				<svelte:fragment slot="icon">
					<WalletSolid class="h-5 w-5" />
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Orders" href="/orders">
				<svelte:fragment slot="icon">
					<ReceiptSolid class="h-5 w-5" />
				</svelte:fragment>
			</SidebarItem>
		</SidebarGroup>
		<SidebarGroup border>
			<SidebarItem label="Help" href="/help">
				<svelte:fragment slot="icon">
					<LifeBuoySolid class="h-5 w-5" />
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Documentation" href="https://docs.rainlang.xyz/intro">
				<svelte:fragment slot="icon">
					<OpenBookSolid class="h-5 w-5" />
				</svelte:fragment>
			</SidebarItem>
		</SidebarGroup>
		<SidebarGroup border class="flex justify-end">
			<DarkMode />
		</SidebarGroup>
	</SidebarWrapper>
</Sidebar>
