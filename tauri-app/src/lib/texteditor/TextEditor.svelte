<script lang="ts">
	import { lightTheme } from './themes';
	import CodeMirror from 'svelte-codemirror-editor';
	import { forkParseDotrain } from './forkParse';
	import { RainlangExtension, type LanguageServicesConfig, RainDocument, type Problem, MetaStore } from 'codemirror-rainlang';

	// @TODO - reactive vars, for fork url and block number
	export let forkUrl: string;
	export let forkBlockNumber: number;
	$: forkUrl;
	$: forkBlockNumber;

	// the fork calback fn
	const callback = async(dotrain: RainDocument): Promise<Problem[]> => {
		return forkParseDotrain(dotrain, forkUrl, forkBlockNumber);
	}

	const metaStore = new MetaStore(false);
	// extension config
	const config: LanguageServicesConfig = {
		hover: true,
		completion: true,
		callback,
		metaStore
	};
	const rainlangCodemirror = new RainlangExtension(config);

	// get the codemirror view, state, etc from the plugin once the codemirror instance is runnings
	// plugin is undefined until the codemirror ext is instantiated
	$: plugin = rainlangCodemirror.plugin;

	// open/close problem panel - it also has default hotkey cmd + shift + m
	// import { openLintPanel, closeLintPanel } from "@codemirror/lint";
	// if (plugin) openLintPanel(plugin?.view)

	// the extension theme, light/dark
	const theme = lightTheme;

	// initial dotrain string
	export let dotrain_string: string = "/* start writing your expression */";

</script>

<div class="h-full flex-grow">
	<CodeMirror
		bind:value={dotrain_string}
		theme={theme}
		styles={{
			'&': {
				flexGrow: 1,
				height: '100%'
			}
		}}
    useTab={true}
    tabSize={2}
		extensions={[
			rainlangCodemirror
		]}
	/>
</div>

<style lang="postcss" global>
	.codemirror-wrapper {
		display: flex;
		height: 100%;
	}
</style>