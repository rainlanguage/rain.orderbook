import { getContext } from 'svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

const GUI_CONTEXT_KEY = 'gui-context';

export function useGui(): DotrainOrderGui {
	return getContext<DotrainOrderGui>(GUI_CONTEXT_KEY);
}
