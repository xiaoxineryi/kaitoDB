use crate::SIZE;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use std::borrow::{Borrow, BorrowMut};
use std::fs::read;
use crate::Disk::Disk::DiskManager;

pub struct Buffer{
    pub file_name:String,
    pub is_dirty:bool,
    pub page_id:u32,
    pub buffer:[u8;SIZE]
}

impl Default for Buffer{
    fn default() -> Self {
        Buffer{
            file_name: "".to_string(),
            is_dirty: false,
            page_id: 0,
            buffer: [0u8;SIZE]
        }
    }
}

type BS = Vec<Rc<RefCell<Buffer>>>;

pub struct BufferPool{
    buffers:BS,
    pool_size:u32,
    ptr:u32
}

struct BufferPoolBuilder{
    pool_size:u32
}

impl Default for BufferPoolBuilder{
    fn default() -> Self {
        BufferPoolBuilder{
            pool_size:100
        }
    }
}

impl BufferPoolBuilder{
    //获得 缓冲池创建器
    pub fn new() -> BufferPoolBuilder{
        BufferPoolBuilder::default()
    }
    //修改缓冲池大小
    pub fn with_size(mut self,size:u32)->BufferPoolBuilder{
        self.pool_size = size;
        self
    }
    // 创建缓冲池
    pub fn build(self)->BufferPool{
        BufferPool{
            buffers: vec![Rc::new(RefCell::new(
                Buffer::default()
            ))],
            pool_size: self.pool_size,
            ptr: 0
        }
    }
}

impl Default for BufferPool{
    fn default() -> Self {
        BufferPoolBuilder::default().build()
    }
}

pub struct BufferReference{
    pub buffer:Rc<RefCell<Buffer>>
}
impl BufferPool{
    pub fn get_page(&mut self,file_name:&str,page_id:u32)->BufferReference{
        // 如果需要的页在缓存中存在的话，就直接返回
        for buffer in self.buffers.as_slice(){
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let e = d.borrow();
            if e.file_name == file_name && e.page_id == page_id {
                return BufferReference{
                    buffer: buffer.clone()
                }
            }
        };

        let disk_handler = DiskManager::get_file(file_name);
        let page = disk_handler.get_page(page_id);
        //这里以后要模拟使用策略来进行页面替换
        let d = self.ptr as usize;
        let mut e =self.buffers.get_mut(d).unwrap();
        let s = Buffer{
            file_name: String::from(file_name),
            is_dirty: false,
            page_id,
            buffer: page
        };
        if let Some(v_i) = self.buffers.get_mut(d){
            *v_i = Rc::new(RefCell::new(s));
        }

        let r = BufferReference{
            buffer: self.buffers.get(d).unwrap().clone()
        };
        self.ptr = (self.ptr + 1) % SIZE as u32;
        r
    }

    pub fn flush_page(&mut self,index:usize){
        let page = self.buffers.get(index).unwrap();
        let d:&RefCell<Buffer> = (*page).borrow();
        let buffer_info = d.borrow();
        let disk_handler = DiskManager::get_file(buffer_info.file_name.as_str());
        disk_handler.flush_page(buffer_info.page_id,buffer_info.buffer);
    }
}

