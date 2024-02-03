<script lang="ts">
	import { darkTheme } from './themes/dark';
  import { lightTheme } from './themes/light';
	import CodeMirror from 'svelte-codemirror-editor';
	import { forkParseDotrain } from './forkParse';
  import { rpcUrl } from '$lib/stores/settings';
	import { openLintPanel, closeLintPanel } from "@codemirror/lint";
	import { RainlangExtension, type LanguageServicesConfig, RainDocument, type Problem, MetaStore } from 'codemirror-rainlang';

	// reactive vars, for fork url and block number
	export let forkUrl: string;
	$: forkUrl;
	const blockNumber = 100000; // should be set to some convenient value

	// the fork calback fn
	const callback = async(dotrain: RainDocument): Promise<Problem[]> => {
		return forkParseDotrain(dotrain, forkUrl, blockNumber);
	}

	// extension config
	const config: LanguageServicesConfig = {
		hover: true,
		completion: true,
		callback
	};
	const metaStore = new MetaStore(false);
	const rainlangCodemirror = new RainlangExtension(config, metaStore);
	
	// get the codemirror view, state, etc from the plugin once the codemirror instance is runnings
	// ley view = plugin.view;
	$: plugin = rainlangCodemirror.plugin;

	// the extension theme
	const theme = lightTheme;

</script>

<div class="h-full flex-grow">
	<CodeMirror
		value={""}
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