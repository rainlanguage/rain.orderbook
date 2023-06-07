// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;  
import "lib/forge-std/src/Test.sol";
import "lib/forge-std/src/console.sol"; 
import "lib/rain.interface.orderbook/src/IOrderBookV2.sol" ; 
import "rain.interface.interpreter/IExpressionDeployerV1.sol";
import {
    DeployerDiscoverableMetaV1ConstructionConfig
} from "lib/rain.interface.interpreter/src/deployerDiscoverable/DeployerDiscoverableMetaV1.sol"; 

import "../src/OrderBook.sol" ; 
import "../src/mock/Rainterpreter.sol" ; 
import "../src/mock/RainterpreterStore.sol" ; 
import "../src/mock/RainterpreterExpressionDeployer.sol" ; 
import "../src/util/ERC20Token.sol" ; 


contract OrderBookTest is Test{ 

    Rainterpreter interpreter ; 
    RainterpreterStore store ; 
    RainterpreterExpressionDeployer deployer ; 
    OrderBook orderBook ;  

    ERC20Token tokenA ; 
    ERC20Token tokenB ;  

    function deployTokens() internal {
       tokenA = new ERC20Token("USDC" , "USDC") ; 
       tokenB = new ERC20Token("USDT" , "USDT") ; 
    }

    function deployOrderBook() internal {  

      // Deploy Interpreter
      interpreter = new Rainterpreter() ; 

      // Deploy Store
      store = new RainterpreterStore() ;  

      // Deploy ExpressionDeployer
      deployer = new RainterpreterExpressionDeployer() ;  

      // Build Constructor Config
      bytes memory meta = hex"56ffc3fc82109c33f1e1544157a70144fc15e7c6e9ae9c65a636fd165b1bc51c" ; 
      DeployerDiscoverableMetaV1ConstructionConfig memory config = DeployerDiscoverableMetaV1ConstructionConfig(
        address(deployer) , 
        meta
      );  

      // Deploy orderBook
      orderBook = new OrderBook(config) ;  
    }

    function setUp() public { 
      // Deploy OrderBook 
      deployOrderBook() ;  
      // Deploy Tokens 
      deployTokens() ; 
    }    

    function testAddOrder(address alice) public { 

      vm.assume(alice != address(0)) ;  
      vm.prank(alice); 
      tokenA.mint(alice,1) ; 
      assertEq(tokenA.balanceOf(alice),1);  

    } 



    

}