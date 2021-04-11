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
    pub buffer:[u8;SIZE],
    pub is_used:bool
}

impl Default for Buffer{
    fn default() -> Self {
        Buffer{
            file_name: "".to_string(),
            is_dirty: false,
            page_id: 0,
            buffer: [0u8;SIZE],
            is_used :false
        }
    }
}

type BS = Vec<Rc<RefCell<Buffer>>>;

pub struct BufferPool{
    buffers:BS,
    pool_size:u32,
    ptr:u32
}


pub struct BufferPoolBuilder{
    pool_size:u32
}

impl Default for BufferPoolBuilder{
     fn default() -> Self {
        BufferPoolBuilder{
            pool_size:5
        }
    }
}

pub struct Replace{}
impl Replace{
    pub fn replace(&self,buffers:BS)->usize{
        unimplemented!()
    }
}

impl BufferPoolBuilder{
    //获得 缓冲池创建器
    pub fn new() -> BufferPoolBuilder{
        BufferPoolBuilder::default()
    }
    pub fn get(size:u32) -> Self{
        BufferPoolBuilder{
            pool_size: size
        }
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
            ));self.pool_size as usize],
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
    pub fn make_dirty(&mut self,file_name:&str,page_id:u32){
        // 如果需要的页在缓存中存在的话，就直接返回
        for buffer in self.buffers.as_slice(){
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let mut e = d.borrow_mut();
            if e.file_name == file_name && e.page_id == page_id {
                e.is_dirty = true;
                break;
            }
        }
    }


    fn change_page_clock(&mut self,file_name:&str, page_id:u32,page:&[u8;SIZE],is_used_n:bool,is_dirty_n:bool)->Option<BufferReference>{
        let mut mut_buffer = self.buffers.clone();
        let mut buffer_ref:Option<BufferReference> = None ;
        for (index,buffer) in self.buffers.as_slice().iter().enumerate(){
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let e = d.borrow();
            if e.is_used == is_used_n && e.is_dirty == is_dirty_n {
                let s = Buffer{
                    file_name: String::from(file_name),
                    is_dirty: false,
                    page_id,
                    buffer: *page,
                    is_used: true
                };
                let return_buffer = Rc::new(RefCell::new(s));
                if let Some(v_i) = mut_buffer.get_mut(index){
                    *v_i = return_buffer.clone();
                }
                buffer_ref = Some(BufferReference{
                    buffer: buffer.clone()
                });
                break;
            }
        };

        if buffer_ref.is_some() {
            self.buffers = mut_buffer;
            for buffer in self.buffers.as_slice().iter() {
                let d:&RefCell<Buffer> = (*buffer).borrow();
                let e = d.borrow();
                println!("the file_name is {},the page_id is {},the is_dirty is {},the is_used is {}",
                         e.file_name,e.page_id,e.is_dirty,e.is_used);
            }
            println!("finish clock");
            return buffer_ref;
        }else {
            return None;
        }
    }

    pub fn get_page_clock(&mut self, file_name:&str, page_id:u32)->BufferReference{
        //先遍历一遍，找有没有存在在缓冲里
        let mut mut_buffer = self.buffers.clone();
        let mut buffer_ref:Option<BufferReference> = None ;
        for (index,buffer) in self.buffers.as_slice().iter().enumerate(){
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let e = d.borrow();
            if e.file_name == file_name && e.page_id == page_id {
                buffer_ref = Some(BufferReference{
                    buffer: buffer.clone()
                });
                break
            }
        };
        if buffer_ref.is_some() {
            self.buffers = mut_buffer;
            for buffer in self.buffers.as_slice().iter() {
                let d:&RefCell<Buffer> = (*buffer).borrow();
                let e = d.borrow();
                println!("the file_name is {},the page_id is {},the is_dirty is {},the is_used is {}",
                         e.file_name,e.page_id,e.is_dirty,e.is_used);
            }
            println!("finish clock");
            return buffer_ref.unwrap();
        }
        // 要是找不到的话，要从磁盘中读
        let disk_handler = DiskManager::get_file(file_name);
        let page = disk_handler.get_page(page_id);
        //先遍历一遍 找没有使用的
        let buffer_unused = self.change_page_clock(file_name,page_id,&page,false,false);
        if buffer_unused.is_some() {
            return buffer_unused.unwrap();
        }
        //再遍历一遍，找使用但是没有被修改的
        let buffer_used = self.change_page_clock(file_name,page_id,&page,true,false);
        if buffer_used.is_some() {
            return buffer_used.unwrap();
        };
        //最后遍历一遍，直接替换一个被使用的
        let buffer_final = self.change_page_clock(file_name,page_id,&page,true,true);
        buffer_final.unwrap()
    }


    pub fn get_page_lru(&mut self, file_name:&str, page_id:u32) ->BufferReference{
        // 如果需要的页在缓存中存在的话，就直接返回
        let mut mut_buffer = self.buffers.clone();
        let k = self.buffers.clone();
        let mut buffer_ref:Option<BufferReference> = None ;
        for (index,buffer) in self.buffers.as_slice().iter().enumerate(){
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let e = d.borrow();
            if e.file_name == file_name && e.page_id == page_id {
                for i in index..(self.pool_size-1) as usize {
                    if let Some(v_i) = mut_buffer.get_mut(i){
                        *v_i = k[i+1].clone();
                    }
                }
                if let Some(v_i) = mut_buffer.get_mut((self.pool_size-1)as usize){
                    *v_i = buffer.clone();
                }
                buffer_ref = Some(BufferReference{
                    buffer: buffer.clone()
                });
            }
        };
        if buffer_ref.is_some() {
            self.buffers = mut_buffer;
            for buffer in self.buffers.as_slice().iter() {
                let d:&RefCell<Buffer> = (*buffer).borrow();
                let e = d.borrow();
                println!("the file_name is {},the page_id is {}",e.file_name,e.page_id);
            }
            println!("finish lru");
            return buffer_ref.unwrap();
        }
        let disk_handler = DiskManager::get_file(file_name);
        let page = disk_handler.get_page(page_id);
        //这里以后要模拟使用策略来进行页面替换

        for i in 0..(self.pool_size - 1) as usize {
            if let Some(v_i) = self.buffers.get_mut(i){
                *v_i = k[i+1].clone();
            }
        }

        let s = Buffer{
            file_name: String::from(file_name),
            is_dirty: false,
            page_id,
            buffer: page,
            is_used: false
        };
        let return_buffer = Rc::new(RefCell::new(s));
        if let Some(v_i) = self.buffers.get_mut((self.pool_size -1) as usize){
            *v_i = return_buffer.clone();
        }

        let r = BufferReference{
            buffer: return_buffer.clone()
        };
        for buffer in self.buffers.as_slice().iter() {
            let d:&RefCell<Buffer> = (*buffer).borrow();
            let e = d.borrow();
            println!("the file_name is {},the page_id is {}",e.file_name,e.page_id);
        }
        println!("finish lru");
        r
    }

    pub fn flush_page(&mut self,index:usize){
        let page = self.buffers.get(index).unwrap();
        let d:&RefCell<Buffer> = (*page).borrow();
        let mut buffer_info = d.borrow_mut();
        let disk_handler = DiskManager::get_file(buffer_info.file_name.as_str());
        buffer_info.is_used = false;
        buffer_info.is_dirty = false;
        disk_handler.flush_page(buffer_info.page_id,buffer_info.buffer);
    }

    // pub fn exit(&mut self){
    //     let mut index = 0;
    //     for buffer in self.buffers.as_slice() {
    //         let d:&RefCell<Buffer> = (*buffer).borrow();
    //         let e = d.borrow();
    //         if e.file_name == "".to_string(){
    //
    //         }
    //         index += 1;
    //     }
    // }
}






