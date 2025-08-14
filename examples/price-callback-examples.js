/**
 * Price Callback Examples for DotrainOrderGui
 * 
 * This file demonstrates various implementations of price callback functions
 * that can be used with the DotrainOrderGui price ratio functionality.
 */

// ============================================================================
// Basic Price Callback Examples
// ============================================================================

/**
 * Simple SushiSwap price callback
 * Uses SushiSwap's API to fetch current token prices
 */
const sushiSwapPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  try {
    const response = await fetch(
      `https://api.sushi.com/price/${inputTokenAddress}/${outputTokenAddress}`
    );
    
    if (!response.ok) {
      throw new Error(`SushiSwap API returned ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    if (!data.price) {
      throw new Error('No price data returned from SushiSwap API');
    }
    
    return data.price.toString();
  } catch (error) {
    throw new Error(`SushiSwap price fetch failed: ${error.message}`);
  }
};

/**
 * 1inch price callback with proper amount scaling
 * Uses 1inch API to get accurate swap quotes
 */
const oneInchPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  try {
    // Use 1 ETH worth of tokens for quote (18 decimals)
    const amount = '1000000000000000000';
    
    const response = await fetch(
      `https://api.1inch.io/v5.0/1/quote?` +
      `fromTokenAddress=${inputTokenAddress}&` +
      `toTokenAddress=${outputTokenAddress}&` +
      `amount=${amount}`
    );
    
    if (!response.ok) {
      throw new Error(`1inch API returned ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    if (!data.toTokenAmount || !data.fromTokenAmount) {
      throw new Error('Invalid quote data from 1inch API');
    }
    
    // Calculate price ratio
    const ratio = parseFloat(data.toTokenAmount) / parseFloat(data.fromTokenAmount);
    return ratio.toString();
  } catch (error) {
    throw new Error(`1inch price fetch failed: ${error.message}`);
  }
};

// ============================================================================
// Advanced Price Callback Examples
// ============================================================================

/**
 * Multi-source price callback with fallback
 * Tries multiple price sources and falls back if one fails
 */
const robustPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  const sources = [
    {
      name: 'SushiSwap',
      fetch: () => sushiSwapPriceCallback(inputTokenAddress, outputTokenAddress)
    },
    {
      name: '1inch',
      fetch: () => oneInchPriceCallback(inputTokenAddress, outputTokenAddress)
    },
    {
      name: 'CoinGecko',
      fetch: () => coinGeckoPriceCallback(inputTokenAddress, outputTokenAddress)
    }
  ];
  
  const errors = [];
  
  for (const source of sources) {
    try {
      console.log(`Trying ${source.name} for price data...`);
      const price = await source.fetch();
      console.log(`✅ ${source.name} returned price: ${price}`);
      return price;
    } catch (error) {
      console.warn(`❌ ${source.name} failed: ${error.message}`);
      errors.push(`${source.name}: ${error.message}`);
    }
  }
  
  throw new Error(`All price sources failed:\n${errors.join('\n')}`);
};

/**
 * Cached price callback to avoid rate limiting
 * Caches prices for a short duration to improve performance
 */
const cachedPriceCallback = (() => {
  const cache = new Map();
  const CACHE_DURATION = 30000; // 30 seconds
  
  return async (inputTokenAddress, outputTokenAddress) => {
    const cacheKey = `${inputTokenAddress}-${outputTokenAddress}`;
    const cached = cache.get(cacheKey);
    
    // Return cached price if still valid
    if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
      console.log(`📦 Using cached price for ${cacheKey}: ${cached.price}`);
      return cached.price;
    }
    
    try {
      // Fetch fresh price
      const price = await robustPriceCallback(inputTokenAddress, outputTokenAddress);
      
      // Cache the result
      cache.set(cacheKey, {
        price,
        timestamp: Date.now()
      });
      
      console.log(`💾 Cached new price for ${cacheKey}: ${price}`);
      return price;
    } catch (error) {
      // If we have a stale cached price, use it as fallback
      if (cached) {
        console.warn(`⚠️ Using stale cached price due to fetch error: ${error.message}`);
        return cached.price;
      }
      throw error;
    }
  };
})();

/**
 * CoinGecko price callback (helper for robust callback)
 * Uses CoinGecko's API for price data
 */
const coinGeckoPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  try {
    // Note: CoinGecko requires token IDs, not addresses
    // This is a simplified example - you'd need to map addresses to CoinGecko IDs
    const response = await fetch(
      `https://api.coingecko.com/api/v3/simple/token_price/ethereum?` +
      `contract_addresses=${inputTokenAddress},${outputTokenAddress}&` +
      `vs_currencies=usd`
    );
    
    if (!response.ok) {
      throw new Error(`CoinGecko API returned ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    const inputPrice = data[inputTokenAddress.toLowerCase()]?.usd;
    const outputPrice = data[outputTokenAddress.toLowerCase()]?.usd;
    
    if (!inputPrice || !outputPrice) {
      throw new Error('Price data not available for one or both tokens');
    }
    
    const ratio = inputPrice / outputPrice;
    return ratio.toString();
  } catch (error) {
    throw new Error(`CoinGecko price fetch failed: ${error.message}`);
  }
};

// ============================================================================
// Usage Examples
// ============================================================================

/**
 * Example: Creating DotrainOrderGui with price callback
 */
async function createGuiWithPricing() {
  const dotrainYaml = `
    # Your YAML configuration here
    # Can include \${io-ratio(input, output)} expressions
  `;
  
  try {
    // Create GUI with cached price callback for best performance
    const result = await DotrainOrderGui.newWithDeploymentAndPriceCallback(
      dotrainYaml,
      'auction-dca', // deployment name
      (serializedState) => {
        // State update callback
        localStorage.setItem('orderState', serializedState);
        console.log('State updated and saved');
      },
      cachedPriceCallback // Price callback
    );
    
    if (result.error) {
      console.error('❌ Failed to create GUI:', result.error.readableMsg);
      return null;
    }
    
    console.log('✅ GUI created successfully with price callback support');
    return result.value;
    
  } catch (error) {
    console.error('❌ Unexpected error:', error.message);
    return null;
  }
}

/**
 * Example: Testing price callback independently
 */
async function testPriceCallback() {
  const USDC = '0xA0b86a33E6441b8e776f89d2b5B977c737C5e5b5';
  const WETH = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
  
  console.log('Testing price callbacks...');
  
  try {
    // Test simple callback
    console.log('\n--- Testing SushiSwap callback ---');
    const sushiPrice = await sushiSwapPriceCallback(USDC, WETH);
    console.log(`SushiSwap USDC/WETH price: ${sushiPrice}`);
    
    // Test robust callback
    console.log('\n--- Testing robust callback ---');
    const robustPrice = await robustPriceCallback(USDC, WETH);
    console.log(`Robust USDC/WETH price: ${robustPrice}`);
    
    // Test cached callback
    console.log('\n--- Testing cached callback ---');
    const cachedPrice1 = await cachedPriceCallback(USDC, WETH);
    console.log(`Cached USDC/WETH price (first call): ${cachedPrice1}`);
    
    const cachedPrice2 = await cachedPriceCallback(USDC, WETH);
    console.log(`Cached USDC/WETH price (second call): ${cachedPrice2}`);
    
  } catch (error) {
    console.error('❌ Price callback test failed:', error.message);
  }
}

/**
 * Example: Error handling for price callbacks
 */
async function handlePriceErrors() {
  const invalidAddress = '0xinvalid';
  const WETH = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
  
  try {
    await robustPriceCallback(invalidAddress, WETH);
  } catch (error) {
    console.error('Expected error for invalid address:', error.message);
    
    // Show user-friendly error message
    const userMessage = error.message.includes('price sources failed') 
      ? 'Unable to fetch current market price. Please check your internet connection and try again.'
      : 'Invalid token configuration. Please verify token addresses.';
      
    console.log('User-friendly message:', userMessage);
  }
}

// ============================================================================
// Export for use in other modules
// ============================================================================

// For ES6 modules
export {
  sushiSwapPriceCallback,
  oneInchPriceCallback,
  robustPriceCallback,
  cachedPriceCallback,
  createGuiWithPricing,
  testPriceCallback,
  handlePriceErrors
};

// For CommonJS
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    sushiSwapPriceCallback,
    oneInchPriceCallback,
    robustPriceCallback,
    cachedPriceCallback,
    createGuiWithPricing,
    testPriceCallback,
    handlePriceErrors
  };
}

// ============================================================================
// Browser usage example
// ============================================================================

// If running in browser, you can test the callbacks
if (typeof window !== 'undefined') {
  window.priceCallbackExamples = {
    sushiSwapPriceCallback,
    oneInchPriceCallback,
    robustPriceCallback,
    cachedPriceCallback,
    createGuiWithPricing,
    testPriceCallback,
    handlePriceErrors
  };
  
  console.log('Price callback examples loaded. Try:');
  console.log('- priceCallbackExamples.testPriceCallback()');
  console.log('- priceCallbackExamples.createGuiWithPricing()');
}
