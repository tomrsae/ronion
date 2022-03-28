const GROWTH: usize = 5;

pub struct UIDGenerator {
    ids: Vec<bool>,
}

impl UIDGenerator {
    // Returns a new UIDGenerator of the given size
    // The generator is capable of generating identifiers that are unique to its context
    // param init_size: The initial size of the UIDGenerator
    pub fn new(init_size: usize) -> Self {
        Self {
            ids: vec![false; init_size],
        }
    }

    // Returns a new unqiue identifier
    pub fn get_uid(&mut self) -> u32 {
        let pos = self.ids.iter().position(|val| !*val);

        match pos {
            Some(id) => {
                self.ids[id] = true;
                id as u32
            }
            None => {
                self.ids.extend(vec![false; GROWTH]);
                self.get_uid()
            }
        }
    }

    // Clears the given unique identified from the generator, freeing it up for future use
    // param index: The index to clear (ID to free up)
    pub fn clear_uid(&mut self, index: u32) {
        let mut val = self.ids.get_mut(index as usize);
        val = Some(&mut false);
    }
}
