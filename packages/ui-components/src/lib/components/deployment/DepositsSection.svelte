<script lang="ts">
	import DepositInput from './DepositInput.svelte';
	import type { DotrainOrderGui, GuiDepositCfg } from '@rainlanguage/orderbook/js_api';

	export let allDepositFields: GuiDepositCfg[];
	export let gui: DotrainOrderGui;
</script>

{#each allDepositFields as deposit}
	<DepositInput bind:deposit {gui} />
{/each}
