const GROWTH: usize = 5;

pub struct UIDGenerator {
    ids: Vec<bool>
}

impl UIDGenerator {
    pub fn new(init_size: usize) -> Self {
        Self {
            ids: vec![false; init_size]
        }
    }

    pub fn get_uid(&self) -> u32 {
        let pos = self.ids.iter().position(|val| !*val);

        match pos {
            Some(id) => id as u32,
            None => {
                self.ids.extend(vec![false; GROWTH]);
                self.get_uid()
            }
        }
    }

    pub fn clear_uid(&self, index: u32) {
        let mut val = self.ids.get_mut(index as usize);
        val = Some(&mut false);
    }
}