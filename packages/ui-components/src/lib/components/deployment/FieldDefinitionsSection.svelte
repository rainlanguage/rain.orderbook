<script lang="ts">
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import { type DotrainOrderGui, type GuiFieldDefinition } from '@rainlanguage/orderbook/js_api';

	export let allFieldDefinitions: GuiFieldDefinition[];
	export let gui: DotrainOrderGui;
	export let open: boolean;

	$: currentOpen = open;

	$: console.log('open in section', currentOpen);
	$: console.log('open in section', open);
</script>

{#each allFieldDefinitions as fieldDefinition}
	<FieldDefinitionInput {fieldDefinition} {gui} open={currentOpen} />
{/each}
