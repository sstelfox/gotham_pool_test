extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;

extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};

use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use redis::Commands;

use std::sync::{Arc, Mutex};

#[derive(Clone, StateData)]
struct RedisPool {
    pool: Arc<Mutex<Pool<RedisConnectionManager>>>,
}

impl RedisPool {
    fn new() -> Self {
        let db_manager = RedisConnectionManager::new("redis://127.0.0.1:6379/0").unwrap();

        let pool = r2d2::Pool::builder()
            .build(db_manager)
            .expect("Unable to connect redis");

        Self {
            pool: Arc::new(Mutex::new(pool)),
        }
    }

    fn pool_handle(&self) -> Pool<RedisConnectionManager> {
        self.pool.lock().unwrap().clone()
    }
}

fn direct_handler(state: State) -> (State, String) {
    let message = {
        let redis_client = redis::Client::open("redis://127.0.0.1:6379/0").unwrap();
        let redis_conn = redis_client.get_connection().unwrap();

        let counter : usize = redis_conn.incr("test_counter", 1).unwrap();

        format!("Direct handler #{}\n", counter)
    };

    (state, message)
}

fn fixed_handler(state: State) -> (State, String) {
    let message = {
        let counter : usize = 0;
        format!("Fixed handler #{}\n", counter)
    };

    (state, message)
}

fn pool_handler(state: State) -> (State, String) {
    let message = {
        let redis_conn = RedisPool::borrow_from(&state).pool_handle().try_get().unwrap();
        let counter : usize = redis_conn.incr("test_counter", 1).unwrap();

        format!("Pool handler #{}\n", counter)
    };

    (state, message)
}

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(StateMiddleware::new(RedisPool::new()))
            .build()
    );

    build_router(chain, pipelines, |route| {
        route.get("/").to(fixed_handler);
        route.get("/direct").to(direct_handler);
        route.get("/pool").to(pool_handler);
    })
}

fn main() {
    gotham::start("127.0.0.1:9292", router())
}
