//! copy from [https://github.com/lionsoul2014/ip2region/tree/master/binding/rust/xdb](https://github.com/lionsoul2014/ip2region/tree/master/binding/rust/xdb)
mod ip_value;
pub use self::ip_value::ToUIntIP;
pub mod searcher;
pub use searcher::{search_by_ip, searcher_init};
