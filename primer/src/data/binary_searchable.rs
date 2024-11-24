pub trait BinarySearchable<T: PartialEq + PartialOrd> {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&T>;
    fn binary_search(&self, item: &T, mut low: usize, mut high: usize) -> Result<usize, usize> {
        assert!(high >= low);
        assert!(high < self.len() || high == self.len() && high == 0);

        if self.len() == 0 {
            return Err(0);
        }

        while low <= high {
            let i = low + (high - low) / 2;
            let item_at_i = self.get(i).unwrap();

            if item_at_i == item {
                return Ok(i);
            } else if item_at_i < item {
                low = i + 1;
            } else {
                high = match i {
                    0 => return Err(0),
                    i => i - 1,
                }
            }
        }
        Err(low)
    }
}
