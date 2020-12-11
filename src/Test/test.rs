#[cfg(test)]
mod tests{
    use super::*;
    use crate::Record::Record::{Format, TableManager, TableHandler};
    use crate::BufferPool::BufferPool::BufferPool;


    #[test]
    fn test_create_table(){
        let format_a = Format{ attr_name: "userID".to_string(), attr_type: 1 };
        let format_b = Format{ attr_name: "userName".to_string(), attr_type: 2 };
        let attr_vec = vec![format_a,format_b];
        let table_handler = TableManager::create_table("user",attr_vec,20);

        let table_h = TableManager::open_table("user");
        assert_eq!(table_handler.attr_num,2);

    }

    #[test]
    fn test_open_table(){
        let table_handler = TableManager::open_table("user");
        println!("{}",table_handler.attr_num);
    }

    #[test]
    fn test_buffer_read_and_flush(){
        let mut buffer_pool = BufferPool::default();
        let buffer_ref = buffer_pool.get_page("1.txt",0);
        let buffer = buffer_ref.buffer.clone();
        {
            //读
            let  e = buffer.borrow();
            for b in e.buffer.iter() {
                print!("{}",b);
            }
            assert_eq!(e.buffer[5],0);
        }

        {
            // 写
            let mut e = buffer.borrow_mut();
            e.buffer[5] = 1;
        }

        buffer_pool.flush_page(0);

        {
            //读
            let  e = buffer.borrow();
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
}