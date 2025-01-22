import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	console.log('params', params);
	const owner = 'rainlanguage';
	const repo = 'rain.strategies';
	const path = 'strategies/dev';
	try {
		const response = await fetch(
			`https://api.github.com/repos/${owner}/${repo}/contents/${path}/${params.filename}`
		);
		const data = await response.json();
		const dotrainResponse = await fetch(data.download_url);
		const dotrain = await dotrainResponse.text();
		if (!response.ok) {
			throw new Error(`HTTP error - status: ${response.status}`);
		}
		return { dotrain, strategyName: params.filename, strategyUrl: data.download_url, deployment: params.deployment };
	} catch (e) {
		return {
			error: 'Error loading strategy',
			errorDetails: e instanceof Error ? e.message : 'Unknown error'
		};
	}
};
