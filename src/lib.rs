use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::time::SystemTime;

//Order
// TODO:: Find a way to attach these enums to the Order struct only and not a global enums
// TODO:: Fix the string types in this project, currently all of them are owned strings

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Copy)]
pub enum OrderType {
    Mkt,
    Limit,
}

#[derive(Clone, Debug, Copy)]
pub enum EventType {
    New,
    Rpl,
    Cxl,
}
#[derive(Debug, Clone)]
pub struct MatchingResult {
    matched_order_ids_: Vec<String>,
    executed_qty_: i32,
    executed_price_: f32,
}

impl MatchingResult {
    fn default() -> Self {
        MatchingResult {
            matched_order_ids_: Vec::new(),
            executed_qty_: 0,
            executed_price_: 0.0,
        }
    }
}

impl PartialEq for MatchingResult {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for MatchingResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MatchingResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.executed_qty_
            .partial_cmp(&other.executed_qty_)
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for MatchingResult {}

#[derive(Clone, Debug)]
pub struct Order {
    id_: String,
    symbol_: String,
    qty_: i32,
    price_: f32,
    entry_time_: SystemTime,
    side_: OrderSide,
    type_: OrderType,
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
    side_: OrderSide,
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
        self.compare(p_other)
    }
    // TODO:: Figure out a way to store sell in opposite order of buy
}

impl Eq for Level {}

impl Level {
    fn compare(&self, p_other: &Self) -> Ordering {
        //assert!(self.side_ == p_other.side_);
        match self.side_ {
            OrderSide::Buy => p_other
                .price_
                .partial_cmp(&self.price_)
                .unwrap_or(Ordering::Equal),
            OrderSide::Sell => self
                .price_
                .partial_cmp(&p_other.price_)
                .unwrap_or(Ordering::Equal),
        }
    }

    fn from_order(p_order: &Order) -> Self {
        let new_level = Level {
            price_: p_order.price_,
            orders_: BTreeSet::new(),
            side_: p_order.side_,
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
        println!("Order id {:?} added into {:?}", p_order.id_, self);
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        //match the qty
        //step 1: get copy of first order
        //step 2: if p_order qty is == first order then remove that order and return total qty of current order as Ok
        //step 2: if p_order qty is < first order then replace that order with qty = that order qty - p_order qty
        //step 3: of p_order qty is > first order then remove first order, p_order.qty -= first_order.qty_  and repeat from step 1

        let mut executed_qty = 0;
        let mut remaining_qty = p_order.qty_;
        let mut avg_matched_price = 0.0;

        println!("Executing {remaining_qty}");
        let mut result = MatchingResult::default();
        while remaining_qty > 0 && !self.orders_.is_empty() {
            let first_order_if_any = self.orders_.first();
            match first_order_if_any {
                None => {
                    return Ok(None);
                }

                Some(first_order) => {
                    result.matched_order_ids_.push(first_order.id_.to_owned());
                    let mut copy_of_first_order = (*first_order).clone();
                    println!("match found order:\n\t {:?}", copy_of_first_order);

                    if remaining_qty == copy_of_first_order.qty_ {
                        //remove order and return exec qty
                        copy_of_first_order.qty_ = 0;
                        executed_qty = remaining_qty;
                        remaining_qty = 0;
                        avg_matched_price += copy_of_first_order.price_ * executed_qty as f32;
                        self.orders_.pop_first();
                    } else if remaining_qty < copy_of_first_order.qty_ {
                        executed_qty += remaining_qty;
                        copy_of_first_order.qty_ -= remaining_qty;
                        avg_matched_price += copy_of_first_order.price_ * remaining_qty as f32;
                        remaining_qty = 0;
                        println!("{executed_qty}  is executed and {remaining_qty} remaining, inplace order\n\t {:?}", copy_of_first_order);
                        self.orders_.replace(copy_of_first_order);
                        println!("Orders in level after this match:\n\t {:?}", self.orders_);
                    } else if remaining_qty > copy_of_first_order.qty_ {
                        let being_executed = copy_of_first_order.qty_;
                        copy_of_first_order.qty_ -= 0;
                        executed_qty += being_executed;
                        remaining_qty -= being_executed;
                        println!(
                            "{being_executed}  is being executed and {remaining_qty} remaining."
                        );
                        avg_matched_price += copy_of_first_order.price_ * being_executed as f32;
                        self.orders_.pop_first();
                    }
                }
            }
        }
        result.executed_qty_ = executed_qty;
        if executed_qty > 0 {
            result.executed_price_ = avg_matched_price / executed_qty as f32;
        }
        return Ok(Some(result));
    }
}

#[derive(Debug)]
struct OrderBook {
    bids_: BTreeSet<Level>,
    asks_: BTreeSet<Level>,
}

impl OrderBook {
    fn add_first_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        match p_order.side_ {
            OrderSide::Buy => {
                self.bids_.insert(Level::from_first_order(p_order));
                return Ok(None);
            }

            OrderSide::Sell => {
                self.asks_.insert(Level::from_first_order(p_order));
                return Ok(None);
            }
        }
    }

    fn get_level_match(&self, p_input_order: &Order) -> Option<&Level> {
        match p_input_order.side_ {
            OrderSide::Buy => match p_input_order.type_ {
                OrderType::Mkt => {
                    return self.asks_.first();
                }
                OrderType::Limit => {
                    return self.asks_.get(&Level::from_order(p_input_order));
                }
            },
            OrderSide::Sell => match p_input_order.type_ {
                OrderType::Mkt => {
                    return self.bids_.first();
                }
                OrderType::Limit => {
                    return self.bids_.get(&Level::from_order(p_input_order));
                }
            },
        }
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        let found_level = self.get_level_match(p_order);
        match found_level {
            None => {
                return Ok(None);
            }
            Some(matched_level) => {
                println!("Matched to {:?}", matched_level);
                let mut copy_of_matched_level = (*matched_level).clone();
                let match_result = copy_of_matched_level.match_order(p_order)?;

                if match_result.is_none() {
                    return Ok(None);
                }

                match p_order.side_ {
                    OrderSide::Buy => {
                        if copy_of_matched_level.orders_.is_empty() {
                            self.asks_.remove(&copy_of_matched_level);
                        } else {
                            self.asks_.replace(copy_of_matched_level);
                        }
                    }
                    OrderSide::Sell => {
                        if copy_of_matched_level.orders_.is_empty() {
                            self.bids_.remove(&copy_of_matched_level);
                        } else {
                            self.bids_.replace(copy_of_matched_level);
                        }
                    }
                }
                println!("After match {:?}", self);
                return Ok(match_result);
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
        println!("After add_order {:#?}", self);
    }
}

#[derive(Debug)]
pub struct MatchingEngine {
    order_book_by_symbol_: HashMap<String, OrderBook>,
}

impl MatchingEngine {
    pub fn process_new_order(
        &mut self,
        p_order: &mut Order,
    ) -> Result<Option<MatchingResult>, String> {
        let order_book_or_error = self.get_book_by_symbol(&p_order.symbol_);
        match order_book_or_error {
            None => {
                if let Some(new_order_book) = self.add_order_book(&p_order.symbol_) {
                    return new_order_book.add_first_order(p_order);
                }
                return Err(String::from(
                    "Failed to add first order in a order book of symbol {p_order.symbol_}",
                ));
            }

            Some(order_book) => {
                let matching_result_or_none = order_book.match_order(p_order)?;
                match matching_result_or_none {
                    None => {
                        order_book.add_order(p_order);
                        println!("After add {:?}", self);
                        return Ok(None);
                    }
                    Some(match_result) => {
                        p_order.qty_ -= match_result.executed_qty_;
                        if p_order.qty_ > 0 {
                            order_book.add_order(p_order);
                        }
                        println!(
                            "Match result: {:?}, order qty: {} ",
                            match_result, p_order.qty_
                        );
                        return Ok(Some(match_result));
                    }
                }
            }
        }
    }

    pub fn contains(&self, p_symbol: &String) -> bool {
        self.order_book_by_symbol_.contains_key(p_symbol)
    }

    fn get_book_by_symbol(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {
        if let Some(mutable_order) = self.order_book_by_symbol_.get_mut(p_symbol) {
            return Some(mutable_order);
        }
        return None;
    }

    fn add_order_book(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {
        let new_order_book = OrderBook {
            bids_: BTreeSet::new(),
            asks_: BTreeSet::new(),
        };

        self.order_book_by_symbol_
            .insert(p_symbol.to_owned(), new_order_book);
        return self.order_book_by_symbol_.get_mut(p_symbol);
    }
}

pub fn process_event(
    p_event_type: EventType,
    p_order: &mut Order,
    p_order_book_collection: &mut MatchingEngine,
) -> Result<Option<MatchingResult>, String> {
    match p_event_type {
        EventType::New => {
            println!("\nNew Order, received:\n\t {:?}", p_order);
            return p_order_book_collection.process_new_order(p_order);
        }

        EventType::Rpl => {
            // TODO::
            return Ok(None);
        }

        EventType::Cxl => {
            // TODO::
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn validate_result(
        p_result: &Result<Option<MatchingResult>, String>,
        p_exp_exec_qty: i32,
        p_exp_exec_price: f32,
        p_matched_order_ids: Option<&Vec<String>>,
    ) {
        match p_result {
            Ok(match_result_or_none) => match match_result_or_none {
                None => {
                    assert!(p_exp_exec_qty == 0);
                }
                Some(match_result) => {
                    assert_eq!(match_result.executed_qty_, p_exp_exec_qty);
                    match p_matched_order_ids {
                        None => {
                            assert!(match_result.matched_order_ids_.is_empty());
                        }
                        Some(matched_ord_ids) => {
                            assert_eq!(match_result.executed_price_, p_exp_exec_price);
                            assert_eq!(&match_result.matched_order_ids_, matched_ord_ids);
                        }
                    }
                }
            },
            Err(error_msg) => {
                println!("process event failed with error {error_msg}");
                assert!(false);
            }
        }
    }

    #[test]
    fn create_first_order() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);
    }

    #[test]
    fn qty_match_simple_order() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        matched_order_ids.push("1".to_string());
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        matched_order_ids.clear();
        matched_order_ids.push("3".to_string());
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));
    }

    #[test]
    fn qty_macth_test_partial_match() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 added to book, exected 0;
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 partially executed, 100 buy left in book
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 executed, 100 sell id 3 left in book
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 executed, nothing left in book
        matched_order_ids.clear();
        matched_order_ids.push("3".to_string());
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("5"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 buy added in book, nothing executed
        matched_order_ids.clear();
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("6"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 buy sell matched, nothin left in book
        matched_order_ids.push("5".to_string());
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_simple() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
        //TODO::include match price, matched order ids in result

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //another 200@100 added into book but not matched
        validate_result(&result, 0, 0.0, None);
    }

    #[test]
    fn mkt_order_match_time() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched 200@100
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
        //TODO::include match price, matched order ids in result

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched 200@100
        matched_order_ids.clear();
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_price() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@101 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 101.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 101.0, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_price_sell_buy() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 102.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@101 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 101.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 101.0, Some(&matched_order_ids));
        //TODO::include match price, matched order ids in result

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 102.0, Some(&matched_order_ids));
    }
}
