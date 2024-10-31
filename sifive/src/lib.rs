#![no_std]

// lib.rs or main.rs

// Import the necessary modules and crates
extern crate embassy_time_driver;

// Import your time driver
mod time_driver;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
