import { REGISTRY_URL } from '$lib/constants';
import type { LayoutLoad } from './$types';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';
import type { DotrainRegistry } from '@rainlanguage/orderbook';

/**
 * Type defining the structure of the load function's return value,
 * including registry information and validation results.
 */
type LoadResult = {
	registryFromUrl: string;
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	registry: DotrainRegistry | null;
	error: string | null;
};

export const load: LayoutLoad<LoadResult> = async ({ url, parent }) => {
	const parentData = await parent();
	const registryFromParent = (parentData as { registryUrl?: string }).registryUrl;
	const registryFromUrl = url.searchParams.get('registry') || registryFromParent || REGISTRY_URL;
	const registry = (parentData as { registry?: DotrainRegistry | null }).registry ?? null;

	if (!registry) {
		return {
			registryFromUrl,
			validOrders: [],
			invalidOrders: [],
			registry,
			error: 'Registry not loaded'
		};
	}

	try {
		const ordersMap = registry.orders as unknown as Map<string, string>;
		const orderDetails = registry.getAllOrderDetails();
		if (orderDetails.error) {
			return {
				registryFromUrl,
				validOrders: [],
				invalidOrders: [],
				registry,
				error: orderDetails.error.readableMsg ?? orderDetails.error.msg
			};
		}

		const validOrders: ValidOrderDetail[] = Array.from(orderDetails.value.valid.entries()).map(
			([name, details]) => ({
				name,
				dotrain: ordersMap.get(name) ?? '',
				details
			})
		);
		const invalidOrders: InvalidOrderDetail[] = Array.from(
			orderDetails.value.invalid.entries()
		).map(([name, err]) => ({
			name,
			error: err.readableMsg ?? err.msg
		}));

		return {
			registryFromUrl,
			validOrders,
			invalidOrders,
			registry,
			error: null
		};
	} catch (error: unknown) {
		return {
			registryFromUrl,
			validOrders: [],
			invalidOrders: [],
			registry,
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
