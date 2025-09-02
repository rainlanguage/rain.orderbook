import type { LayoutLoad } from '../$types';

export const load: LayoutLoad = async ({ params }) => {
    const { deploymentKey } = params;
    return {
        pageName: deploymentKey
    };
};
