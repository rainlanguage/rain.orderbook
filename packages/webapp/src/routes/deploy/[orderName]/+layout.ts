import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ params }) => {
    const { orderName } = params;
    return {
        orderName,
        pageName: orderName
    };
};
