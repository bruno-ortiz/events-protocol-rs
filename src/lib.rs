mod errors;
mod events;
mod handlers;
mod processor;
mod store;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
