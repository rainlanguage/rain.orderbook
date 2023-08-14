
use std::cmp::Ordering;
use cursive::align::HAlign;
use cursive::traits::*;
use cursive::views::{Dialog, TextView, CircularFocus};
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};

use crate::subgraph::showorder::{get_order_details_display, OrdersDetails};

#[derive(Clone, Debug)]
struct Order {
    id: String,
    owner: String
} 

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum OrderColumn {
    OrderId,
    OrderOwner

}

#[allow(dead_code)]
impl OrderColumn {
    fn as_str(&self) -> &str {
        match *self {
            OrderColumn::OrderId => "Order ID",
            OrderColumn::OrderOwner => "Order Owner",

        }
    }
}

impl TableViewItem<OrderColumn> for Order {
    fn to_column(&self, column: OrderColumn) -> String {
        match column {
            OrderColumn::OrderId => self.id.to_string(),
            OrderColumn::OrderOwner => self.owner.to_string()

        }
    }

    fn cmp(&self, _other: &Self, column: OrderColumn) -> Ordering
    where
        Self: Sized,
    {
        match column { 
            _ => Ordering::Equal
        } 
    }
}

#[derive(Clone, Debug)]
struct Vaults {
    token : String ,
    balance : String
} 

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum VaultsColumn{
    Token,
    Balance
}

#[allow(dead_code)]
impl VaultsColumn {
    fn as_str(&self) -> &str {
        match *self {
            VaultsColumn::Token => "Token",
            VaultsColumn::Balance => "Balance",
        }
    }
} 

impl TableViewItem<VaultsColumn> for Vaults {
    fn to_column(&self, column: VaultsColumn) -> String {
        match column {
            VaultsColumn::Token => self.token.to_string(),
            VaultsColumn::Balance => self.balance.to_string(),
        }
    }

    fn cmp(&self, _other: &Self, column: VaultsColumn) -> Ordering
    where
        Self: Sized,
    {
        match column { 
            _ => Ordering::Equal
        } 
    }
}

#[derive(Clone, Debug)]
struct TakeOrders{
    input_token : String , 
    input_amount : String ,
    output_token : String ,
    output_amount : String ,
    transaction_id : String
} 

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum TakeOrdersColumn{
    InputToken,
    InputAmount,
    OutputToken,
    OutputAmount,
    TransactionId
} 

#[allow(dead_code)]
impl TakeOrdersColumn {
    fn as_str(&self) -> &str {
        match *self {
            TakeOrdersColumn::InputToken => "Input Token",
            TakeOrdersColumn::InputAmount => "Input Amount",
            TakeOrdersColumn::OutputToken => "Output Token",
            TakeOrdersColumn::OutputAmount => "Output Amount",
            TakeOrdersColumn::TransactionId => "Transaction Id",
        }
    }
} 

impl TableViewItem<TakeOrdersColumn> for TakeOrders {
    fn to_column(&self, column: TakeOrdersColumn) -> String {
        match column {
            TakeOrdersColumn::InputToken => self.input_token.to_string(),
            TakeOrdersColumn::InputAmount => self.input_amount.to_string(),
            TakeOrdersColumn::OutputToken => self.output_token.to_string(),
            TakeOrdersColumn::OutputAmount => self.output_amount.to_string(),
            TakeOrdersColumn::TransactionId => self.transaction_id.to_string(),
        }
    }

    fn cmp(&self, _other: &Self, column: TakeOrdersColumn) -> Ordering
    where
        Self: Sized,
    {
        match column { 
            _ => Ordering::Equal
        } 
        
    }
}



pub async fn view_orders(sg_uri : String) {

    let mut siv = cursive::default();
      
    let orders = get_order_details_display(sg_uri).await.unwrap() ; 

    // let input_vault_table = 
    let mut table = get_main_table() ;
    let items = get_main_table_items(orders.clone()) ;
    table.set_items(items); 

    table.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| { 

        let value = siv
            .call_on_name("Orders", move |table: &mut TableView<Order, OrderColumn>| { 
                let order_id = &table.borrow_item(index).unwrap().id ;
                format!("Order Id : {:?}", order_id)
            })
            .unwrap(); 
        
        // Move orders value
        let orders_input = orders.clone() ;
        let orders_output = orders.clone() ;
        let take_orders = orders.clone() ;


        siv.add_layer(
            Dialog::around(TextView::new(value))
            .title("Order Details")
            .button("View Input Vault Balances", move |s| { 
                let input_vault_table = get_input_vault_table(orders_input.clone(),index) ;
                s.add_layer(
                    Dialog::around(
                        input_vault_table.with_name("Input Vault").min_size((100, 100))
                    )
                            .title("Input Vaults")
                            .button("Back", move |s| {
                                s.pop_layer();
                            })
                            .wrap_with(CircularFocus::new)
                            .wrap_tab()        
                    );
                
            })
            .button("View Output Vault Balances", move |s| {
                let output_vault_table = get_output_vault_table(orders_output.clone(),index) ;
                s.add_layer(
                    Dialog::around(
                        output_vault_table.with_name("Output Vault").min_size((100, 100))
                    )
                            .title("Output Vaults")
                            .button("Back", move |s| {
                                s.pop_layer();
                            })
                            .wrap_with(CircularFocus::new)
                            .wrap_tab()        
                    );
            })
            .button("View Take Orders", move |s| {
                let take_orders_table = get_take_orders_table(take_orders.clone(),index) ;
                s.add_layer(
                    Dialog::around(
                        take_orders_table.with_name("Take Orders").min_size((100, 100))
                    )
                            .title("Take Orders")
                            .button("Back", move |s| {
                                s.pop_layer();
                            })
                            .wrap_with(CircularFocus::new)
                            .wrap_tab()        
                    );
            })
            .button("Close Menu", move |s| {
                s.pop_layer();
            })
            .wrap_with(CircularFocus::new)
        ) ;  


    }) ; 

    siv.add_layer(
        Dialog::around(
            table.with_name("Orders").min_size((100, 100))
        )
                .title("Order List")
                .button("Exit", move |s| {
                    s.quit();
                })
                .wrap_with(CircularFocus::new)
                .wrap_tab()        
        );
    siv.run(); 
        
}  

fn get_take_orders_table(
    orders : Vec<OrdersDetails> ,
    index : usize
) -> TableView<TakeOrders, TakeOrdersColumn> { 

    let mut take_orders_table = TableView::<TakeOrders, TakeOrdersColumn>::new()
    .column(TakeOrdersColumn::TransactionId, "Transaction Id", |c| {
        c.align(HAlign::Center)
            .width_percent(10)
    })
    .column(TakeOrdersColumn::InputToken, "Sold", |c| {
        c.align(HAlign::Center)
            .width_percent(12)
    })
    .column(TakeOrdersColumn::InputAmount, "Sell Amount", |c| {
        c.align(HAlign::Center)
            .width_percent(33)
    })
    .column(TakeOrdersColumn::OutputToken, "Bought", |c| {
        c.align(HAlign::Center)
            .width_percent(12)
    })
    .column(TakeOrdersColumn::OutputAmount, "Buy Amount", |c| {
        c.align(HAlign::Center)
            .width_percent(33)
    }) ; 

    let mut items: Vec<TakeOrders> = Vec::new() ;
    for (i,order) in orders.iter().enumerate() {
        if i == index {
            for take_order in &order.take_orders { 
                items.push( TakeOrders { 
                    input_token : take_order.input_token.clone() ,
                    input_amount : take_order.input_amount.clone() ,
                    output_token : take_order.output_token.clone() ,
                    output_amount : take_order.output_amount.clone() ,
                    transaction_id : take_order.transaction_id.clone()
                 }) ;

            }
        }
    } 

    take_orders_table.set_items(items) ;
    take_orders_table

}

fn get_input_vault_table(
    orders : Vec<OrdersDetails> ,
    index : usize
) -> TableView<Vaults, VaultsColumn> { 

    let mut input_vault_table = TableView::<Vaults, VaultsColumn>::new()
    .column(VaultsColumn::Token, "Token", |c| c.width_percent(20))
    .column(VaultsColumn::Balance, "Balance", |c| {
        c.align(HAlign::Center)
            .width_percent(80)
    }) ;  

    let mut items: Vec<Vaults> = Vec::new() ;
    for (i,order) in orders.iter().enumerate() {
        if i == index {
            for vault in &order.input_vaults { 

                items.push(Vaults { token: vault.token.clone(), balance: vault.balance.clone() }) ;
            }
        }
    } 

    input_vault_table.set_items(items) ;
    input_vault_table

}  

fn get_output_vault_table(
    orders : Vec<OrdersDetails> ,
    index : usize
) -> TableView<Vaults, VaultsColumn> { 

    let mut output_vault_table = TableView::<Vaults, VaultsColumn>::new()
    .column(VaultsColumn::Token, "Token", |c| c.width_percent(20))
    .column(VaultsColumn::Balance, "Balance", |c| {
        c.align(HAlign::Center)
            .width_percent(80)
    }) ;  

    let mut items: Vec<Vaults> = Vec::new() ;
    for (i,order) in orders.iter().enumerate() {
        if i == index {
            for vault in &order.output_vaults { 

                items.push(Vaults { token: vault.token.clone(), balance: vault.balance.clone() }) ;
            }
        }
    } 

    output_vault_table.set_items(items) ;
    output_vault_table

} 


fn get_main_table() -> TableView<Order, OrderColumn>{
    let table: TableView<Order, OrderColumn> = TableView::<Order, OrderColumn>::new()
    .column(OrderColumn::OrderId, "ID", |c| c.width_percent(50))
    .column(OrderColumn::OrderOwner, "Owner", |c| {
        c.align(HAlign::Center)
            .width_percent(50)
    }) ;
    table
}

fn get_main_table_items(orders : Vec<OrdersDetails>) -> Vec<Order>{  
    let mut items = Vec::new();
    for order in orders {   
        items.push(Order {
            id : order.id,
            owner : order.owner,
        });
    }  
    items
}