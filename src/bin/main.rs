extern crate async_test;

fn main() {
  let addr = "127.0.0.1:3000";
  async_test::run(addr);
}
