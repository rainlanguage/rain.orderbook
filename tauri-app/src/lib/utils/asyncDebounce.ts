import { debounce } from "lodash";
import { get, writable } from "svelte/store";

/**
 * Creates a debounced asynchronous function that calls the original function
 * only once after the last call within a specified wait time.
 *
 * @template T - The type of the arguments passed to the original function.
 * @template R - The type of the result returned by the original function.
 * @param {(...args: T) => Promise<R>} fn - The original function to be debounced.
 * @param {number} wait - The wait time in milliseconds before invoking the debounced function.
 * @param {(result: R) => void} onSuccess - The success callback function to be invoked with the result.
 * @param {(error: unknown) => void} onError - The error callback function to be invoked with the error.
 * @returns {(...args: T) => void} - The debounced function.
 */
export function createDebouncedAsyncFn<T extends unknown[], R>(
    fn: (...args: T) => Promise<R>,
    wait: number,
    onSuccess: (result: R) => void,
    onError: (error: unknown) => void
): (...args: T) => void {
    let currentArgs: T | undefined;

    const executeFn = async (args: T) => {
        const argsNotEqual = (args: T, currentArgs?: T) => JSON.stringify(currentArgs) !== JSON.stringify(args)
        try {
            const result = await fn(...args);
            if (argsNotEqual(args, currentArgs)) return
            onSuccess(result);
        } catch (error) {
            if (argsNotEqual(args, currentArgs)) return
            onError(error);
        }
    };

    const debouncedFn = debounce((...args: T) => {
        return executeFn(args);
    }, wait);

    return (...args: T): void => {
        currentArgs = args;
        debouncedFn(...args);
    };
}


/**
 * Creates a debounced asynchronous function that calls the original function
 * only once after the last call within a specified wait time.
 *
 * The function returns a debounced function along with writable stores for the result and error.
 *
 * @template T - The type of the arguments passed to the original function.
 * @template R - The type of the result returned by the original function.
 * @param {(...args: T) => Promise<R>} fn - The original function to be debounced.
 * @param {number} wait - The wait time in milliseconds before invoking the debounced function.
 * @returns {{ debouncedFn: (...args: T) => void, result: import('svelte/store').Writable<R | undefined>, error: import('svelte/store').Writable<unknown | undefined> }} - The debounced function and the result and error stores.
 */
export function useDebouncedFn<T extends unknown[], R>(fn: (...args: T) => Promise<R>, wait: number) {
    const result = writable<R | undefined>(undefined);
    const error = writable<unknown | undefined>(undefined);

    const debouncedFn = createDebouncedAsyncFn<T, R>(
        fn,
        wait,
        (res: R) => {
            result.set(res);
            error.set(undefined); // Reset error store on successful execution
        },
        (err: unknown) => {
            error.set(err);
            result.set(undefined); // Reset result store on error
        }
    );

    return { debouncedFn, result, error };
}

if (import.meta.vitest) {
    const { it, expect, vi } = import.meta.vitest

    it('creates a debounced async function that only calls the original function once after the last call', async () => {
        let callCount = 0;
        const debouncedFn = createDebouncedAsyncFn(async () => {
            callCount++;
        }, 100, () => { }, () => { });

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 50));

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150));

        expect(callCount).toEqual(1);

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 50));

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150));

        expect(callCount).toEqual(2);
    });

    it('calls the onSuccess callback with the result of the original function', async () => {
        const onSuccess = vi.fn();
        const onError = vi.fn();

        const debouncedFn = createDebouncedAsyncFn(async (value: number) => {
            return value * 2;
        }, 100, onSuccess, onError);

        debouncedFn(5);

        await new Promise((resolve) => setTimeout(resolve, 150)); // Wait for debounce and async function

        expect(onSuccess).toHaveBeenCalledWith(10);
    });

    it('calls the onError callback if the original function throws an error', async () => {
        const onSuccess = vi.fn();
        const onError = vi.fn();

        const debouncedFn = createDebouncedAsyncFn(async () => {
            throw new Error('Test error');
        }, 100, onSuccess, onError);

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150)); // Wait for debounce and async function

        expect(onError).toHaveBeenCalledWith(expect.any(Error));
        expect(onError.mock.calls[0][0]).toHaveProperty('message', 'Test error');
    });

    it('passes the arguments to the original function', async () => {
        const onSuccess = vi.fn();
        const onError = vi.fn();

        const debouncedFn = createDebouncedAsyncFn(async (value: number) => {
            return value * 2;
        }, 100, onSuccess, onError);

        debouncedFn(5);

        await new Promise((resolve) => setTimeout(resolve, 150)); // Wait for debounce and async function

        expect(onSuccess).toHaveBeenCalledWith(10);
    });

    // tests for useDebouncedFn
    it('creates a debounced async function that only calls the original function once after the last call', async () => {
        let callCount = 0;
        const { debouncedFn } = useDebouncedFn(async () => {
            callCount++;
        }, 100);

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 50));

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150));

        expect(callCount).toEqual(1);

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 50));

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150));

        expect(callCount).toEqual(2);
    });

    it('calls the onSuccess callback with the result of the original function', async () => {
        const { debouncedFn, result } = useDebouncedFn(async (value: number) => {
            return value * 2;
        }, 100);

        debouncedFn(5);

        await new Promise((resolve) => setTimeout(resolve, 150)); // Wait for debounce and async function

        expect(get(result)).toEqual(10);
    });

    it('calls the onError callback if the original function throws an error', async () => {
        const { debouncedFn, error } = useDebouncedFn(async () => {
            throw new Error('Test error');
        }, 100);

        debouncedFn();

        await new Promise((resolve) => setTimeout(resolve, 150)); // Wait for debounce and async function

        expect(get(error)).toHaveProperty('message', 'Test error');
    });
}
