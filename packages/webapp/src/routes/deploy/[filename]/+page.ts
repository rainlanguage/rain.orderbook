import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, url }) => {
	const state = url.searchParams.get('state');
	return {
		filename: params.filename,
		state: state ?? null
	};
};