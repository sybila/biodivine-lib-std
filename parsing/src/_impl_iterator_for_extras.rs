use crate::Extras;

impl Extras<'_> {
    // Create a new `Extras` iterator with declared number of skipped elements.
    pub fn new(meta: &Vec<String>, skip: usize) -> Extras {
        return Extras {
            meta,
            skip,
            index: 0,
        };
    }
}

impl<'a> Iterator for Extras<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        return self
            .meta
            .get(self.index + self.skip - 1)
            .map(|i| i.as_str());
    }
}
