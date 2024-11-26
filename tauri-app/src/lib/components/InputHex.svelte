<script lang="ts">
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { fromHex, toHex } from 'viem';
  import { HEX_INPUT_REGEX } from '@rainlanguage/ui-components';

  let valueRaw: string = '';
  export let value: bigint | undefined;
  export let required = true;

  $: {
    if (value !== undefined) {
      valueRaw = toHex(value);
    }
  }

  const maskOptions = {
    // hexadecimal string, optionally starting with 0x
    mask: HEX_INPUT_REGEX,
    lazy: false,
  };

  function complete({ detail }: { detail: InputMask }) {
    valueRaw = detail.value;

    if (detail.unmaskedValue.length === 0) {
      value = 0n;
    } else {
      let valuePrefixed = detail.unmaskedValue;
      if (detail.unmaskedValue.substring(0, 2) !== '0x') {
        valuePrefixed = `0x${valuePrefixed}`;
      }
      try {
        value = fromHex(valuePrefixed as `0x${string}`, 'bigint');
        // eslint-disable-next-line no-empty
      } catch (_e) {}
    }
  }
</script>

<input
  {required}
  type="text"
  value={valueRaw}
  class="block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 dark:focus:border-primary-500 dark:focus:ring-primary-500 rtl:text-right"
  use:imask={maskOptions}
  on:complete={complete}
/>
