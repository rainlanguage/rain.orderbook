import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { debounce } from 'lodash';

export function handleUpdateGuiState(gui: DotrainOrderGui) {
	pushGuiStateToUrlHistory(gui);
}

const pushGuiStateToUrlHistory = debounce((gui: DotrainOrderGui) => {
	const serializedState = gui.serializeState();
	if (serializedState) {
		window.history.pushState({}, '', `?state=${serializedState}`);
	}
}, 1000);
