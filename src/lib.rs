use std::time::SystemTime;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::cmp::Ordering;

//Order
//TODO:: Find a way to attach these enums to the Order struct only and not a global enums

#[derive(Clone, Debug, Copy)]
enum OrderSide {
    Buy,
    Sell
}

#[derive(Clone, Debug, Copy)]
enum OrderType {
    Mkt,
    Limit
}

#[derive(Clone, Debug, Copy)]
enum EventType {
    New,
    Rpl,
    Cxl
}

#[derive(Clone, Debug)]
struct Order {
    id_: String,
    symbol_: String,
    qty_: i32,
    price_: f32,
    entry_time_: SystemTime,
    side_: OrderSide,
    type_: OrderType
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.entry_time_
            .partial_cmp(&other.entry_time_)
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for Order {}

#[derive(Clone, Debug)]
struct Level {
    orders_: BTreeSet<Order>,
    price_: f32,
    side_ : OrderSide
}

impl PartialOrd for Level {
    fn partial_cmp(&self, p_other: &Self) -> Option<Ordering> {
        Some(self.cmp(p_other))
    }
}

impl PartialEq for Level {
    fn eq(&self, p_other: &Self) -> bool {
        self.cmp(p_other) == Ordering::Equal
    }
}

impl Ord for Level {
    fn cmp(&self, p_other: &Self) -> Ordering {
        self.price_
            .partial_cmp(&p_other.price_)
            .unwrap_or(Ordering::Equal)
    }
    // TODO:: Figure out a way to store sell in opposite order of buy
}

impl Eq for Level {}

impl Level {
    
    fn from_order(p_order: &Order) -> Self {
        let new_level = Level {
            price_: p_order.price_,
            orders_: BTreeSet::new(),
            side_: p_order.side_
        };
        new_level
    }

    fn from_first_order(p_order: &Order) -> Self {
        let mut new_level = Level::from_order(p_order);
        new_level.add_order(p_order);
        new_level
    }

    fn add_order(&mut self, p_order: &Order) {
        self.orders_.insert(p_order.to_owned());
        println!("Order id {:?} added into Level {:?}", p_order.id_, self);
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<i32, String> {
        //match the qty
        //step 1: get copy of first order
        //step 2: if p_order qty is == first order then remove that order and return total qty of current order as Ok
        //step 2: if p_order qty is < first order then replace that order with qty = that order qty - p_order qty 
        //step 3: of p_order qty is > first order then remove first order, p_order.qty -= first_order.qty_  and repeat from step 1 

        let mut executed_qty = 0;
        let mut remaining_qty = p_order.qty_;
        
        //println!("Executing new order {} against ");
        while remaining_qty > 0 && !self.orders_.is_empty() {

            let first_order_if_any = self.orders_.first();
            match first_order_if_any {
                None => { 
                    return Ok(executed_qty); 
                }

                Some(first_order) => {
                    let mut copy_of_first_order = (*first_order).clone();
                    if remaining_qty == copy_of_first_order.qty_ {
                        //remove order and return exec qty
                        executed_qty += remaining_qty;
                        self.orders_.pop_first();
                        return Ok(executed_qty);
                    }
                    else if remaining_qty < copy_of_first_order.qty_ {
                        //reduce orderbook order qty and return exec qty
                        executed_qty += remaining_qty;
                        copy_of_first_order.qty_ -= remaining_qty;
                        self.orders_.replace(copy_of_first_order);
                        return Ok(executed_qty);
                    }
                    else {
                        //remove order, increament execqty, decreament remaining qty and move to next order
                        copy_of_first_order.qty_ -= remaining_qty;
                        remaining_qty -= copy_of_first_order.qty_;
                        executed_qty += copy_of_first_order.qty_;
                        self.orders_.pop_first();
                    }
                }
            }
        }
        return Ok(executed_qty);
    }
}

struct OrderBook {
    bids_ : BTreeSet<Level>,
    asks_ : BTreeSet<Level>
}

impl OrderBook {
    fn add_first_order(&mut self, p_order: &mut Order) -> Result<i32, String> {
        match p_order.side_ {
            OrderSide::Buy => {
                self.bids_.insert(Level::from_first_order(p_order));
                return Ok(0);
            }

            OrderSide::Sell => {
                self.asks_.insert(Level::from_first_order(p_order));
                return Ok(0);
            }
        }
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<i32, String> {
        let level = Level::from_order(p_order);
        match p_order.side_ {
            
            OrderSide::Buy => {
                let found_level = self.asks_.get(&level);
                match found_level {
                    None => { 
                        return Ok(0); 
                    }

                    Some(matched_level) => {
                        let mut copy_of_matched_level = (*matched_level).clone();
                        let executed_qty = copy_of_matched_level.match_order(p_order)?;
                        if executed_qty > 0 {
                            self.asks_.replace(copy_of_matched_level);
                        }
                        return Ok(executed_qty);
                    }
                }
            }
            
            OrderSide::Sell => {
                let found_level = self.bids_.get(&level);
                match found_level {
                    None => { 
                        return Ok(0); 
                    }

                    Some(matched_level) => {
                        let mut copy_of_matched_level = (*matched_level).clone();
                        let executed_qty = copy_of_matched_level.match_order(p_order)?;
                        if executed_qty > 0 {
                            self.bids_.replace(copy_of_matched_level);
                        }
                        return Ok(executed_qty);
                    }
                }
            }
        }
    }

    fn add_order(&mut self, p_order: &mut Order) -> Result<i32, String> {
        let mut temp_level = Level::from_order(&p_order);
        match p_order.side_ {
            OrderSide::Buy => {
                let found_level = self.bids_.get(&temp_level);

                match found_level {
                    
                    None => {
                        temp_level.add_order(p_order);
                        self.bids_.insert(temp_level);
                        return Ok(0);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.bids_.replace(copy_of_found_level);
                        return Ok(0);
                    }
                }
            }

            OrderSide::Sell => {
                let found_level = self.asks_.get(&temp_level);

                match found_level {
                    
                    None => {
                        temp_level.add_order(p_order);
                        self.asks_.insert(temp_level);
                        return Ok(0);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.asks_.replace(copy_of_found_level);
                        return Ok(0);
                    }
                }
            }
        }
    }


}

pub struct OrderBookCollection {
    book_by_symbol: HashMap<String, OrderBook>
}

impl OrderBookCollection {

    pub fn contains(&self, p_symbol: &String) -> bool {
        self.book_by_symbol.contains_key(p_symbol)
    }

    pub fn get_book_by_symbol(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {

        if let Some(mutable_order) = self.book_by_symbol.get_mut(p_symbol) {
            return Some(mutable_order);
        }
        return None;
    }

    pub fn add_order_book(&mut self, p_symbol: &String) -> Option<&mut OrderBook>{
        let new_order_book = OrderBook {
            bids_: BTreeSet::new(),
            asks_:BTreeSet::new()
        };

        self.book_by_symbol.insert(p_symbol.to_owned(), new_order_book);
        return self.book_by_symbol.get_mut(p_symbol);
    }

}
 
pub fn process_event(p_event_type: EventType, p_order: &mut Order, p_order_book_collection: &mut OrderBookCollection) -> Result<i32, String>
{
    //Process the event and return exececuted qty on success or error on failure

    match p_event_type {
        EventType::New => {

            //handle if it is first order of this symbol
            println!("New Order , received: {:?}", p_order);
            let order_book_or_error = p_order_book_collection.get_book_by_symbol(&p_order.symbol_);

            match order_book_or_error {
                None => {
                    if let Some(new_order_book) = p_order_book_collection.add_order_book(&p_order.symbol_) {
                        return new_order_book.add_first_order(p_order)
                    }
                    return Err(String::from("Failed to add first order in a order book of symbol {p_order.symbol_}"));
                }

                Some(order_book) => {
                    let executed_qty = order_book.match_order(p_order)?; 

                    if executed_qty > 0 {
                        println!("Ordergot executed :  {:?}, \n executed_qty: {executed_qty}", p_order);
                        return Ok(executed_qty);
                    }
                    println!("Match Not found for order:  {:?}", p_order);
                    return order_book.add_order(p_order);
                }
            }
        }

        EventType::Rpl =>  {
            // TODO::
            return Ok(p_order.qty_);
        }

        EventType::Cxl => {
            // TODO:: 
            return Ok(p_order.qty_);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;


    #[test]
    fn create_first_order() {
        println!("==============================Test create_first_order starts======================================");
        let mut order_book_collection = OrderBookCollection {
            book_by_symbol: HashMap::new()
        };

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(0));
        println!("==============================Test create_first_order ends======================================");
    }

    #[test]
    fn match_simple_order() {
        println!("==============================Test match_simple_order starts======================================");
     
        let mut order_book_collection = OrderBookCollection {
            book_by_symbol: HashMap::new()
        };

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(0));

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(order.qty_));

        
        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(0));

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(order.qty_));

        println!("==============================Test match_simple_order ends======================================");
    }

    #[test]
    fn partial_match_simple_order() {
        println!("==============================Test match_simple_order starts======================================");
     
        let mut order_book_collection = OrderBookCollection {
            book_by_symbol: HashMap::new()
        };

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(0));

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(order.qty_));

        
        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(0));

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(order.qty_));

        let mut order = Order {
            id_: String::from("5"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(100));

        
        let mut order = Order {
            id_: String::from("6"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        assert_eq!(result, Ok(100));


        println!("==============================Test match_simple_order ends======================================");
    }
}
