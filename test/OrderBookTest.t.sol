// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;  
import "forge-std/Test.sol";
import "forge-std/console.sol"; 
import "../src/OrderBook.sol" ; 
import "../src/mock/Rainterpreter.sol" ; 
import "../src/mock/RainterpreterStore.sol" ; 
import "../src/mock/RainterpreterExpressionDeployer.sol" ; 
import "rain.interface.orderbook/IOrderBookV2.sol" ; 
import {
    DeployerDiscoverableMetaV1ConstructionConfig
} from "rain.interface.interpreter/deployerDiscoverable/DeployerDiscoverableMetaV1.sol";

contract OrderBookTest is Test{ 

    Rainterpreter interpreter ; 
    RainterpreterStore store ; 
    RainterpreterExpressionDeployer deployer ; 
    OrderBook orderBook ; 

    function deployOrderBook() internal { 

      // Deploy Interpreter
      interpreter = new Rainterpreter() ; 

      // Deploy Store
      store = new RainterpreterStore() ; 

      // Deploy ExpressionDeployer
      deployer = new RainterpreterExpressionDeployer(
        IInterpreterV1(address(interpreter)) ,
        IInterpreterStoreV1(address(store))
      ) ;   

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

    }  

    

}