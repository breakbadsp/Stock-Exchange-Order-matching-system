use std::time::SystemTime;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::cmp::Ordering;

//Order
//TODO:: Find a way to attach these enums to the Order struct only and not a global enums

#[derive(Clone, Debug, Copy)]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Clone, Debug, Copy)]
pub enum OrderType {
    Mkt,
    Limit
}

#[derive(Clone, Debug, Copy)]
pub enum EventType {
    New,
    Rpl,
    Cxl
}

#[derive(Clone, Debug)]
pub struct Order {
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
    //side_ : OrderSide //use if needed
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
            //side_: p_order.side_
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
                        copy_of_first_order.qty_ = 0;
                        executed_qty = remaining_qty;
                        remaining_qty = 0;
                        self.orders_.pop_first();
                    }
                    else if remaining_qty < copy_of_first_order.qty_ {
                        //reduce orderbook order qty and return exec qty
                        executed_qty += remaining_qty;
                        copy_of_first_order.qty_ -= remaining_qty;
                        remaining_qty = 0;
                        self.orders_.replace(copy_of_first_order);
                    }
                    else if remaining_qty > copy_of_first_order.qty_{
                        let being_executed = copy_of_first_order.qty_;
    
                        copy_of_first_order.qty_ -= being_executed;
                        executed_qty += being_executed;
                        remaining_qty = copy_of_first_order.qty_;
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

    fn get_level_match(&self, p_input_order: &Order) -> Option<&Level> {
        match p_input_order.side_ {
            OrderSide::Buy => {
                match  p_input_order.type_ {
                    OrderType::Mkt => {
                        return self.asks_.first();
                    }
                    OrderType::Limit => {
                        return self.asks_.get(&Level::from_order(p_input_order));
                    }
                }
            }
            OrderSide::Sell => {
                match p_input_order.type_ {
                    OrderType::Mkt => {
                        return self.bids_.first();
                    }
                    OrderType::Limit => {
                        return self.bids_.get(&Level::from_order(p_input_order));
                    }
                }
            }
        }
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<i32, String> {

        let found_level = self.get_level_match(p_order);
        match found_level {
            None => {
                return Ok(0);
            }
            Some(matched_level) => {
                let mut copy_of_matched_level = (*matched_level).clone();
                let executed_qty = copy_of_matched_level.match_order(p_order)?;
                if executed_qty > 0 {
                    match p_order.side_ {                   
                        OrderSide::Buy => {
                            self.asks_.replace(copy_of_matched_level);
                        }
                        OrderSide::Sell => {
                            self.bids_.replace(copy_of_matched_level);
                        }
                    }
                }
                return Ok(executed_qty);
            }
        }
    }

    fn add_order(&mut self, p_order: &mut Order) {
        let mut temp_level = Level::from_order(&p_order);
        match p_order.side_ {
            OrderSide::Buy => {
                let found_level = self.bids_.get(&temp_level);

                match found_level {
                    None => {
                        temp_level.add_order(p_order);
                        self.bids_.insert(temp_level);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.bids_.replace(copy_of_found_level);
                    }
                }
            }

            OrderSide::Sell => {
                let found_level = self.asks_.get(&temp_level);

                match found_level {
                    None => {
                        temp_level.add_order(p_order);
                        self.asks_.insert(temp_level);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.asks_.replace(copy_of_found_level);
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

    pub fn process_new_order(&mut self, p_order: &mut Order) -> Result<i32, String> {

        let order_book_or_error = self.get_book_by_symbol(&p_order.symbol_);
        match order_book_or_error {
            None => {
                if let Some(new_order_book) = self.add_order_book(&p_order.symbol_) {
                    return new_order_book.add_first_order(p_order)
                }
                return Err(String::from("Failed to add first order in a order book of symbol {p_order.symbol_}"));
            }

            Some(order_book) => {
                let executed_qty = order_book.match_order(p_order)?; 
                p_order.qty_ -= executed_qty;
                if p_order.qty_ > 0 {
                    order_book.add_order(p_order);
                }
                println!("Executed Qty: {executed_qty}, order qty: {} ", p_order.qty_);
                return Ok(executed_qty);
            }
        }
    }

    pub fn contains(&self, p_symbol: &String) -> bool {
        self.book_by_symbol.contains_key(p_symbol)
    }

    fn get_book_by_symbol(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {

        if let Some(mutable_order) = self.book_by_symbol.get_mut(p_symbol) {
            return Some(mutable_order);
        }
        return None;
    }

    fn add_order_book(&mut self, p_symbol: &String) -> Option<&mut OrderBook>{
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

            
            println!("\nNew Order, received: {:?}", p_order);
            return p_order_book_collection.process_new_order(p_order);
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
    }

    #[test]
    fn qty_match_simple_order() {     
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
        assert_eq!(result, Ok(200));

        
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
        assert_eq!(result, Ok(200));
    }

    #[test]
    fn qty_macth_test_partial_match() {     
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
        //200 added to book, exected 0;
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
        //100 partially executed, 100 buy left in book
        assert_eq!(result, Ok(100));

        
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
        //100 executed, 100 sell left in book
        assert_eq!(result, Ok(100));

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
        //100 executed, nothing left in book
        assert_eq!(result, Ok(100));

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
        //200 buy added in book, nothing executed
        assert_eq!(result, Ok(0));

        
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
        //200 buy sell matched, nothin left in book
        assert_eq!(result, Ok(200));
    }

    #[test]
    fn mkt_order_match() {
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
        //200@100 buy added to book
        assert_eq!(result, Ok(0));

        
        let mut order = Order {
            id_: String::from("2"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now()
        };
        let result = process_event(EventType::New, &mut order , &mut order_book_collection);
        //mkt matched 
        assert_eq!(result, Ok(200));
        //include match price in result
    }
}
