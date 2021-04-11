pub struct KeyValuePair{
    pub key :u32,
    pub value : u32
}

impl KeyValuePair{
    pub fn new(key:u32,value:u32)->KeyValuePair{
        KeyValuePair{
            key,
            value
        }
    }
}

impl Clone for KeyValuePair {
    fn clone(&self) -> KeyValuePair {
        KeyValuePair {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}