<script lang="ts">
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import WalletConnect from '$lib/components/WalletConnect.svelte';
	import { wagmiConfig, connected } from '$lib/stores/wagmi';
</script>

<div class="flex flex-col justify-center pb-12">
	<DeploymentSteps {wagmiConfig} wagmiConnected={connected}>
		<WalletConnect slot="wallet-connect" />
	</DeploymentSteps>
</div>
