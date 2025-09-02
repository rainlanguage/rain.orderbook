# Price Ratio Functionality in DotrainOrderGui

## Overview

The DotrainOrderGui now supports dynamic price ratio resolution using the `${io-ratio(input, output)}` syntax in YAML configurations. This feature allows strategy writers to reference external price data without hardcoding values, making strategies more dynamic and user-friendly.

## Key Features

- **Dynamic Price Resolution**: Automatically fetch current market prices for token pairs
- **YAML Integration**: Use `${io-ratio(input, output)}` expressions in field names, descriptions, and default values
- **Error Handling**: Comprehensive error handling for network issues, invalid addresses, and callback failures
- **Backward Compatibility**: Existing YAML configurations continue to work without changes

## Usage

### JavaScript Integration

When creating a DotrainOrderGui instance, provide a price callback function:

```javascript
// Define your price callback function
const priceCallback = async (inputTokenAddress, outputTokenAddress) => {
  try {
    // Example using SushiSwap API
    const response = await fetch(
      `https://api.sushi.com/price/${inputTokenAddress}/${outputTokenAddress}`
    );
    const data = await response.json();
    return data.ratio.toString();
  } catch (error) {
    throw new Error(`Failed to fetch price: ${error.message}`);
  }
};

// Create GUI instance with price callback
const result = await DotrainOrderGui.newWithDeploymentAndPriceCallback(
  dotrainYaml,
  "mainnet-deployment",
  (serializedState) => {
    localStorage.setItem('orderState', serializedState);
  },
  priceCallback
);

if (result.error) {
  console.error("Failed to initialize GUI:", result.error.readableMsg);
} else {
  const gui = result.value;
  // Use the GUI instance...
}
```

### YAML Configuration Examples

#### 1. Default Field Values

```yaml
deployments:
  auction-dca:
    name: Auction DCA Strategy
    description: Dollar-cost averaging using auctions
    fields:
      - binding: initial-io
        name: Kickoff ${order.inputs.0.token.symbol} per ${order.outputs.0.token.symbol}
        description: |
          The initial ${order.inputs.0.token.symbol} per ${order.outputs.0.token.symbol} to kickoff the first auction.
          
          This ratio is calculated in the same way as the baseline ratio.
          It must be greater than the baseline ratio, regardless of what you are selling or buying.
        default: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
```

#### 2. Field Names and Descriptions

```yaml
deployments:
  limit-order:
    name: Limit Order
    description: Deploy a limit order
    fields:
      - binding: fixed-io
        name: ${order.inputs.0.token.symbol} per ${order.outputs.0.token.symbol}
        description: |
          Fixed exchange rate (${order.inputs.0.token.symbol} received per 1 ${order.outputs.0.token.symbol} sold).
          
          Current market price: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
          
          Set your desired rate above or below the current market price.
```

#### 3. Complex Strategy Example

```yaml
deployments:
  dsf-strategy:
    name: Dutch Auction Strategy
    description: Automated Dutch auction with dynamic pricing
    fields:
      - binding: start-ratio
        name: Starting ${order.inputs.0.token.symbol}/${order.outputs.0.token.symbol} Ratio
        description: |
          Starting price for the Dutch auction.
          
          Current market rate: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
          
          Recommended: Set 5-10% above market rate for better execution.
        default: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
        
      - binding: end-ratio
        name: Ending ${order.inputs.0.token.symbol}/${order.outputs.0.token.symbol} Ratio
        description: |
          Minimum acceptable price for the Dutch auction.
          
          Current market rate: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
          
          Recommended: Set at or slightly below market rate.
```

## Price Callback Implementation

### Requirements

Your price callback function must:

1. **Accept two parameters**: `inputTokenAddress` and `outputTokenAddress` (both strings)
2. **Return a Promise**: That resolves to a string representation of the price ratio
3. **Handle errors**: Throw meaningful error messages for debugging

### Example Implementations

#### Using SushiSwap API

```javascript
const sushiPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  const response = await fetch(
    `https://api.sushi.com/price/${inputTokenAddress}/${outputTokenAddress}`
  );
  
  if (!response.ok) {
    throw new Error(`SushiSwap API error: ${response.status}`);
  }
  
  const data = await response.json();
  return data.price.toString();
};
```

#### Using 1inch API

```javascript
const oneInchPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  const response = await fetch(
    `https://api.1inch.io/v5.0/1/quote?fromTokenAddress=${inputTokenAddress}&toTokenAddress=${outputTokenAddress}&amount=1000000000000000000`
  );
  
  const data = await response.json();
  const ratio = data.toTokenAmount / data.fromTokenAmount;
  return ratio.toString();
};
```

#### Using Multiple Sources with Fallback

```javascript
const robustPriceCallback = async (inputTokenAddress, outputTokenAddress) => {
  const sources = [
    () => fetchFromSushiSwap(inputTokenAddress, outputTokenAddress),
    () => fetchFromUniswap(inputTokenAddress, outputTokenAddress),
    () => fetchFromOneInch(inputTokenAddress, outputTokenAddress),
  ];
  
  for (const source of sources) {
    try {
      return await source();
    } catch (error) {
      console.warn(`Price source failed: ${error.message}`);
    }
  }
  
  throw new Error('All price sources failed');
};
```

## Error Handling

The system provides comprehensive error handling:

### Common Error Types

1. **No Price Callback**: When `${io-ratio(...)}` is used but no callback was provided
2. **Invalid Token Addresses**: When token addresses are empty or malformed
3. **Network Errors**: When the price callback fails due to network issues
4. **Invalid Response**: When the callback returns non-string or empty values
5. **Invalid Expression**: When the `io-ratio` syntax is incorrect

### Error Messages

All errors include user-friendly messages:

```javascript
// Example error handling
try {
  const gui = await DotrainOrderGui.newWithDeploymentAndPriceCallback(
    yaml, deployment, stateCallback, priceCallback
  );
} catch (error) {
  if (error.message.includes('Price callback')) {
    console.error('Price fetching failed:', error.message);
    // Show user-friendly message about network connectivity
  } else {
    console.error('GUI initialization failed:', error.message);
  }
}
```

## Best Practices

### 1. Robust Price Callbacks

- Implement timeout handling
- Use multiple price sources with fallbacks
- Cache prices for a short duration to avoid rate limiting
- Validate token addresses before making API calls

### 2. YAML Configuration

- Always provide fallback values for critical fields
- Use descriptive field names and descriptions
- Test configurations with different token pairs
- Consider rate limiting when using multiple price ratios

### 3. User Experience

- Show loading states while fetching prices
- Provide clear error messages for price fetching failures
- Allow users to manually override price ratios if needed
- Cache successful price fetches to improve performance

## Migration Guide

### From Hardcoded Values

**Before:**
```yaml
- binding: initial-io
  name: Initial USDC per WETH
  default: "1800.0"
```

**After:**
```yaml
- binding: initial-io
  name: Initial ${order.inputs.0.token.symbol} per ${order.outputs.0.token.symbol}
  description: Current market price: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
  default: ${io-ratio(order.inputs.0.address, order.outputs.0.address)}
```

### Updating JavaScript Code

**Before:**
```javascript
const gui = await DotrainOrderGui.newWithDeployment(yaml, deployment, stateCallback);
```

**After:**
```javascript
const gui = await DotrainOrderGui.newWithDeploymentAndPriceCallback(
  yaml, deployment, stateCallback, priceCallback
);
```

## Troubleshooting

### Common Issues

1. **"Price callback not available"**: Ensure you're using `newWithDeploymentAndPriceCallback` instead of `newWithDeployment`
2. **"Invalid io-ratio expression"**: Check the syntax: `${io-ratio(input_path, output_path)}`
3. **"Price callback failed"**: Check network connectivity and API endpoints
4. **Empty price values**: Ensure your callback returns a non-empty string

### Debug Tips

- Test your price callback function independently
- Use browser developer tools to inspect network requests
- Check console for detailed error messages
- Validate token addresses are correct format (0x...)
