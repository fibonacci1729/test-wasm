#[cfg(test)]
mod tests {
    use test_wasm_macro::test_wasm;

    #[test_wasm]
    fn test_add() {
        assert_eq!(2 + 2, 4);
    }

    #[test_wasm]
    fn test_two() {
        assert_eq!(2, 2);
    }
}