import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const owner = 'rainlanguage';
	const repo = 'rain.strategies';
	const path = 'strategies'; // targeting the strategies folder

	try {
		const response = await fetch(
			`https://api.github.com/repos/${owner}/${repo}/contents/${path}`
		);
		const data = await response.json();

		console.log(data);

		// Filter for files and get their raw URLs
		const files = data


		return {
			files
		};
	} catch (error) {
		console.error('Error fetching GitHub files:', error);
		return {
			files: []
		};
	}
};
