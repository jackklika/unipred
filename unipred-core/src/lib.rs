pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct UnipredCore {
    config: String,
}

impl UnipredCore {
    pub fn new(config: String) -> Self {
        Self { config }
    }

    pub fn execute(&self) -> String {
        format!("Executing with config: {}", self.config)
    }
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
