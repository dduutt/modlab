use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct SessionMemory {
    pub holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self {
            holding_registers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_register(&self, addr: u16) -> u16 {
        let map = self.holding_registers.lock().unwrap();
        *map.get(&addr).unwrap_or(&0)
    }

    pub fn set_register(&self, addr: u16, val: u16) {
        let mut map = self.holding_registers.lock().unwrap();
        map.insert(addr, val);
    }

    pub fn get_range(&self, start: u16, count: u16) -> HashMap<u16, u16> {
        let map = self.holding_registers.lock().unwrap();
        let mut res = HashMap::new();
        for addr in start..(start + count) {
            let val = *map.get(&addr).unwrap_or(&0);
            res.insert(addr, val);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_memory_operations() {
        let session = SessionMemory::new();
        assert_eq!(session.get_register(100), 0);

        session.set_register(100, 42);
        assert_eq!(session.get_register(100), 42);

        session.set_register(101, 99);
        let range = session.get_range(100, 3);
        assert_eq!(range.get(&100), Some(&42));
        assert_eq!(range.get(&101), Some(&99));
        assert_eq!(range.get(&102), Some(&0));
    }

    #[test]
    fn test_isolated_session_memories() {
        let session1 = SessionMemory::new();
        let session2 = SessionMemory::new();

        session1.set_register(10, 500);
        assert_eq!(session1.get_register(10), 500);
        assert_eq!(session2.get_register(10), 0);
    }
}
