<script lang="ts">
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import { type DotrainOrderGui, type GuiFieldDefinitionCfg } from '@rainlanguage/orderbook/js_api';

	export let allFieldDefinitions: GuiFieldDefinitionCfg[];
	export let gui: DotrainOrderGui;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
</script>

{#each allFieldDefinitions as fieldDefinition}
	<FieldDefinitionInput {fieldDefinition} {gui} {handleUpdateGuiState} />
{/each}
