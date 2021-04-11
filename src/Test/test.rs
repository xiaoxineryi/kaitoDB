#[cfg(test)]
mod tests{
    use super::*;
    use crate::Record::Record::{Format, TableManager, TableHandler};
    use crate::BufferPool::BufferPool::{BufferPool, Buffer, BufferPoolBuilder};
    use std::convert::TryInto;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::borrow::Borrow;
    use crate::index::bPlusTree::BPlusTree;
    use crate::index::key_value_pair::KeyValuePair;
    use rand::Rng;

    #[test]
    fn test_create_table(){
        let buffer_pool = BufferPool::default();
        let format_a = Format{ attr_name: "userID".to_string(), attr_type: 1 };
        let format_b = Format{ attr_name: "userName".to_string(), attr_type: 2 };
        let attr_vec = vec![format_a,format_b];
        let table_manager = TableManager{ buffer_pool: Rc::new(RefCell::new(buffer_pool)) };
        let table_handler = table_manager.create_table("user",attr_vec,20);

        let table_h = table_manager.open_table("user");
        assert_eq!(table_handler.attr_num,2);
    }
    #[test]
    fn test_open_table(){
        let buffer_pool = BufferPool::default();
        let table_manager = TableManager{ buffer_pool: Rc::new(RefCell::new(buffer_pool)) };
        let table_handler = table_manager.open_table("user");
        println!("属性个数:{}",table_handler.attr_num);
        println!("第一个属性名:{}",table_handler.attr_format[0].attr_name)
    }
    #[test]
    fn test_buffer_read_and_flush(){
        let mut buffer_pool = BufferPool::default();
        let buffer_ref = buffer_pool.get_page_lru("1.txt", 0);
        let buffer = buffer_ref.buffer.clone();
        {
            //读
            let  e = buffer.borrow_mut();
            for b in e.buffer.iter() {
                print!("{}",b);
            }
            assert_eq!(e.buffer[5],1);
        }

        {
            // 写
            let mut e = buffer.borrow_mut();
            e.buffer[5] = 1;
        }

        buffer_pool.flush_page(0);

        {
            //读
            let  e = buffer.borrow_mut();
            for b in e.buffer.iter() {
                print!("{}",b);
            }
            assert_eq!(e.buffer[5],1);
        }
    }
    #[test]
    fn test_str(){
        let s = String::from("你好");
        let e = s.len() as u8;
        let l = s.as_bytes();
        let string = String::from_utf8_lossy(l);
        println!("{} ",string);

        for i in 0..l.len() {
            print!("{} ",l[i]);
        }
        println!("{}",e);
    }
    #[test]
    fn test_parse(){
        let buffer_pool = BufferPool::default();
        let table_manager = TableManager{ buffer_pool: Rc::new(RefCell::new(buffer_pool)) };
        let table_handler = table_manager.open_table("user");
        let mut vec:Vec<u8> = Vec::new();
        vec.push(4);
        let v = 40u32.to_be_bytes();
        for b in v.iter() {
            vec.push(*b);
        }

        vec.push(6);
        let s = "123456";
        for b in s.bytes(){
            vec.push(b);
        }
        table_handler.parse_item(&vec);
    }
    #[test]
    fn test_page_info_read() {
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let table_manager = TableManager { buffer_pool: buffer_pool_rc.clone() };
        let table_handler = table_manager.open_table("user");
        let mut page_handler = table_handler.page_handler;
        for i in 0..1000 {
            let mut vec:Vec<u8> = Vec::new();
            let e = 32u32;
            vec.push(4);
            let v = 22u32.to_be_bytes();
            for b in v.iter() {
                vec.push(*b);
            }
            let s = "小李";
            vec.push(s.len() as u8);
            for b in s.bytes(){
                vec.push(b);
            }
            page_handler.insert_item(vec);
        }

        buffer_pool_rc.borrow_mut().flush_page(1);
        buffer_pool_rc.borrow_mut().flush_page(2);
        buffer_pool_rc.borrow_mut().flush_page(3);
        buffer_pool_rc.borrow_mut().flush_page(4);
    }
    #[test]
    fn test_page_get(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let table_manager = TableManager { buffer_pool: buffer_pool_rc.clone() };
        let table_handler =  table_manager.open_table("user");
        let mut page_handler = table_handler.page_handler;
        let vec = page_handler.get_items_by_page(0);

        let t = table_manager.open_table("user");
        for v in vec.iter() {
            t.parse_item(v);
        }

        // let vec2 = page_handler.get_items_by_page(2);
        let vec3 = page_handler.get_items_by_page(1);
        for v in vec.iter() {
            t.parse_item(v);
        }
    }

    #[test]
    fn test_lru(){
        let mut buffer_pool = BufferPool::default();
        let file_name = "user";
        buffer_pool.get_page_lru(file_name,1);
        buffer_pool.get_page_lru(file_name,3);
        buffer_pool.get_page_lru(file_name,1);
        buffer_pool.get_page_lru(file_name,4);
        buffer_pool.get_page_lru(file_name,5);
        buffer_pool.get_page_lru(file_name,6);
        buffer_pool.get_page_lru(file_name,2);
    }

    #[test]
    fn test_clock(){
        let mut buffer_pool = BufferPool::default();
        // let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let file_name = "user";
        buffer_pool.get_page_clock(file_name,1);
        buffer_pool.get_page_clock(file_name,2);
        buffer_pool.get_page_clock(file_name,3);
        buffer_pool.get_page_clock(file_name,4);
        buffer_pool.get_page_clock(file_name,5);
        buffer_pool.make_dirty(file_name,1);
        buffer_pool.get_page_clock(file_name,6);
    }

    #[test]
    fn test_bufferPool(){
        let log_buffer_pool_builder = BufferPoolBuilder::new().with_size(10);
        let log_buffer_pool = log_buffer_pool_builder.build();
        let sql_buffer_pool_builder = BufferPoolBuilder::new().with_size(20);
        let sql_buffer_pool_builder = sql_buffer_pool_builder.build();
        let buffer_pool_builder = BufferPoolBuilder::new().with_size(5);
        let buffer_pool = buffer_pool_builder.build();
    }

    #[test]
    fn test_uuid(){
        let a = ((5) << 16)+ 1 as u32;
        println!("{}",a >> 16);
        println!("{}",a );
    }


    #[test]
    fn test_create_index(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let b_plus_tree = BPlusTree::create_index("user.index", 20, buffer_pool_rc.clone());
        buffer_pool_rc.borrow_mut().flush_page(4);
    }

    #[test]
    fn test_get_index(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let mut b_plus_tree = BPlusTree::open_index("user.index", buffer_pool_rc);
        println!("{}",b_plus_tree.root.read().unwrap().uuid);
    }
    #[test]
    fn test_insert_key_value(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let mut b_plus_tree = BPlusTree::open_index("user.index", buffer_pool_rc);
        let mut rng = rand::thread_rng();
        for i in 0..10 {
            let u16:u16 = rng.gen();
            let kv = KeyValuePair{ key: i, value: (1 << 16) + u16 as u32 };
            b_plus_tree.insert(kv);
        }

    }
    #[test]
    fn test_find_index(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let mut b_plus_tree = BPlusTree::open_index("user.index", buffer_pool_rc);
        let key_value_pair = b_plus_tree.search(2);
        println!("所查找的uuid是{}",key_value_pair.unwrap().value);
    }

    #[test]
    fn test_delete_index(){
        let mut buffer_pool = BufferPool::default();
        let buffer_pool_rc = Rc::new(RefCell::new(buffer_pool));
        let mut b_plus_tree = BPlusTree::open_index("user.index", buffer_pool_rc);
        let mut key_value_pair = b_plus_tree.search(2);
        println!("所查找的uuid是{}",key_value_pair.unwrap().value);
        b_plus_tree.delete_index(2);
        key_value_pair = b_plus_tree.search(2);
        println!("所查找的uuid是{}",key_value_pair.unwrap().value);
    }


}