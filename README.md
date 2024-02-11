Exchange Order matching system.

Project overview:
  
  Trying to build a matching engine and order book for a stock market exchange.
  You can read more details about it in https://en.wikipedia.org/wiki/Order_matching_system this page
  
  I can think of two important components of this system.
  1. Order book for order storage.
  2. Matching algorithm for matching orders.
  
  There are lot of intresting tasks that are involved in building matching engine.
  One of the most important design decision is to use which matching algorithm.
  Here we will be using price/time mathing algo for our implimentaion.
  
  More about it :
  Price/Time algorithm (or First-in-First-out)
  Motivates to narrow the spread, since by narrowing the spread the limit order is the first in the order queue.
  Discourages other orders to join the queue since a limit order that joins the queue is the last.
  Might be computationally more demanding than Pro-Rata. 
  The reason is that market participants might want to place more small orders in different positions in the order queue, and also tend to "flood" the market, i.e., place limit order in the depth of the market in order to stay in the queue.
  
  One more intresting design decision I am making here is use of Rust as a programming language.
  I am a professional C++ developer and completely aware of the issues with c++.
  This project is my first experience with Rust and I am excited for this.

How to build and use it:
  - Just clone the repo
  - run cargo test


Future features planned to be added:
  - User authentication service so that users can login 
  - User should be able to send orders and receive NOE.
  - Self match protection with client id in each order.
  - Order Processing service which will use the Matching engine lib
  - Front end to create and send orders(this is for testing)
  - FIX engine to Support for FIX protocol so that any Broker can connect and send orders via fix
  - Low latency Market data distribution service
  - 
  - 