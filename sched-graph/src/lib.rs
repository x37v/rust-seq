trait GraphExec {
    fn exec(&mut self, context: &mut impl Context) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
