
use crate::SIZE;
use crate::Disk::Disk::DiskManager;
use std::convert::TryInto;
use std::rc::Rc;
use crate::BufferPool::BufferPool::{BufferPool, Buffer, BufferReference};
use std::cell::RefCell;
use std::borrow::{ Borrow};
use std::fs::read;
use rand::{random, Rng};


const IS_PAGE:u32 = 796*325*4;


struct PageHeader{
    is_page:u32,
    page_number:u32,
    // old_lsn:u64,
    // new_lsn:u64,
    record_number:u16,
    free_size:u16,
}
#[derive(Debug)]
pub struct ItemInfo{
     pub offset:u16,
     pub uuid:u32,
     pub size:u16
}

pub struct PageInfo{
    page_header:PageHeader,
    pub item_map:Vec<ItemInfo>,
}
impl PageInfo{
    pub fn cal_offset(&self)->usize{
        let number = self.item_map.len();
        number * 8 + 12
    }
}

pub struct ItemHandler{
    pub(crate) file_name:String,
    pub(crate) free_page:u32,
    buffer_pool:Rc<RefCell<BufferPool>>
}
pub struct ItemManager{}
impl ItemManager{
    pub fn new_item_handler(file_name:String,size:u32,buffer_pool:Rc<RefCell<BufferPool>>)->ItemHandler{
        ItemHandler{
            file_name,
            free_page: size,
            buffer_pool
        }
    }
}
impl ItemHandler{
    fn judge_page(&self, page:&BufferReference)->Option<PageInfo>{
        let mut e = page.buffer.clone();
        let mut start = 0 as usize;
        let mut b = (*e).borrow_mut();
        let mut buffer = &mut b.buffer;
        let is_page = u32::from_be_bytes(buffer[start..start + 4].try_into().unwrap());
        start += 4;
        if is_page == IS_PAGE {
            //如果这个页已经写好了，那么就直接读取信息并返回
            //获取页号
            let page_number = u32::from_be_bytes(buffer[start..start+4].try_into().unwrap());
            start += 4;
            //获取记录总数
            let record_number = u16::from_be_bytes(buffer[start..start+2].try_into().unwrap());
            start += 2;
            //获取剩余空间大小
            let free_size = u16::from_be_bytes(buffer[start..start+2].try_into().unwrap());
            // 如果剩余空间太小，就直接返回空
            // if free_size < (SIZE as f32 * 0.2) as u16 {
            //     return None;
            // }
            start += 2;
            let mut records:Vec<ItemInfo> = Vec::new();
            for index in 0..record_number  {
                let offset = u16::from_be_bytes(buffer[start..start+2].try_into().unwrap());
                start += 2;
                let uuid = u32::from_be_bytes(buffer[start..start+4].try_into().unwrap());
                start += 4;
                let size = u16::from_be_bytes(buffer[start..start+2].try_into().unwrap());
                start += 2;
                records.push(ItemInfo{
                    offset,
                    uuid,
                    size
                })
            }

            let page_header  = PageHeader{
                is_page,
                page_number,
                record_number,
                free_size
            };
            return Some(PageInfo{
                page_header,
                item_map: records
            });
        }else {
            // 如果页没有写好，那么就需要对页进行初始化
            start = 0;
            //先赋值
            let page_header = PageHeader{
                is_page :IS_PAGE,
                page_number: self.free_page,
                record_number: 0,
                free_size: 4084
            };
            let is_page= IS_PAGE.to_be_bytes();
            for i in 0..is_page.len() {
                buffer[start] = is_page[i];
                start += 1;
            }
            let page_number = self.free_page.to_be_bytes();
            for i in 0..page_number.len() {
                buffer[start] = page_number[i];
                start += 1;
            }
            let record_number = page_header.record_number.to_be_bytes();
            for i in 0..record_number.len() {
                buffer[start] = record_number[i];
                start += 1;
            }
            let free_size = page_header.free_size.to_be_bytes();
            for i in 0..free_size.len() {
                buffer[start] = free_size[i];
                start += 1;
            }
            return Some(PageInfo{
                page_header,
                item_map: vec![]
            });
        };
    }

    pub fn insert_item(&mut self,item:Vec<u8>)->u32{
        let mut p:PageInfo;
        let mut page:BufferReference;
        loop {
            let p_id = self.free_page + 1;
            let b:&RefCell<BufferPool> = self.buffer_pool.borrow();
            let mut buffer_pool = b.borrow_mut();
            page = buffer_pool.get_page_lru(self.file_name.as_str(), p_id);
            p= self.judge_page(&page).unwrap();
            if p.page_header.free_size < (0.2 * SIZE as f32) as u16 {
                // p为0表示这个页满了
                self.free_page += 1;
            }else{
                //这种表示页没有满，所以写将之变为dirty
                buffer_pool.make_dirty(self.file_name.as_str(),self.free_page);
                break
            }
        }

        let mut page_info = p ;
        let mut buffer = &mut (*page.buffer).borrow_mut().buffer;
        let mut offset = SIZE ;
        if !page_info.item_map.is_empty() {
            let last_item = page_info.item_map.last().unwrap();
            offset = (last_item.offset - last_item.size) as usize;
        }
        let mut rng = rand::thread_rng();
        let n16 : u16 = rng.gen();
        let uuid = (self.free_page << 16) + n16 as u32;
        let item_info = ItemInfo{
            offset: offset as u16,
            uuid,
            size: item.len() as u16
        };
        //插入到页中
        let start = offset - item.len();
        for i in 0..item.len() {
            buffer[start + i] = *item.get(i).unwrap();
        }
        // 修改表数据
        let mut head_offset = page_info.cal_offset();
        page_info.page_header.record_number += 1;
        page_info.page_header.free_size -= item.len() as u16;
        let item_offset = item_info.offset.to_be_bytes();
        for i in 0..item_offset.len() {
            buffer[head_offset] = item_offset[i];
            head_offset += 1;
        }
        let item_uuid = item_info.uuid.to_be_bytes();
        for i in 0..item_uuid.len() {
            buffer[head_offset] = item_uuid[i];
            head_offset += 1;
        }
        let item_size = item_info.size.to_be_bytes();
        for i in 0..item_size.len() {
            buffer[head_offset] = item_size[i];
            head_offset += 1;
        }
        // 修改记录数目
        let mut record_offset = 8usize;
        let record_num = page_info.page_header.record_number.to_be_bytes();
        for i in 0..record_num.len() {
            buffer[record_offset] = record_num[i];
            record_offset += 1;
        }
        //最后修改空闲空间
        let mut free_offset = 10usize;
        let free_size = page_info.page_header.free_size.to_be_bytes();
        for i in 0..free_size.len() {
            buffer[free_offset] = free_size[i];
            free_offset += 1;
        };
        item_info.uuid
    }




    // 直接获取一页数据
    pub fn get_items_by_page(&mut self, page_id:u32)->Vec<Vec<u8>>{
        // 这里对page_id 进行+1 因为第一页始终保存数据头信息。
        let p_id = page_id + 1 ;
        let b:&RefCell<BufferPool> = self.buffer_pool.borrow();
        let mut buffer_pool = b.borrow_mut();
        let page = buffer_pool.get_page_lru(self.file_name.as_str(), p_id);
        let page_info = self.judge_page(&page).unwrap();
        let b = page.buffer.clone();
        let buffer = &((*b).borrow().buffer);

        let mut vec:Vec<Vec<u8>> = Vec::new();
        for item_info in page_info.item_map {
            // println!("{:?}",item_info);
            let offset = item_info.offset as usize;
            let size = item_info.size as usize;
            let start = offset - size;
            let mut v = Vec::new();
            for index in start..offset {
                v.push(buffer[index]);
            }
            vec.push(v);
        };
        vec
    }
    // 获取某个数据
    pub fn get_item_by_uuid(&mut self, uuid:u32) ->Option<Vec<u8>>{
        let page_id = (uuid >> 16);
        let p_id = page_id + 1 ;
        let b:&RefCell<BufferPool> = self.buffer_pool.borrow();
        let mut buffer_pool = b.borrow_mut();
        let page = buffer_pool.get_page_lru(self.file_name.as_str(), p_id);
        let page_info = self.judge_page(&page).unwrap();
        let b = page.buffer.clone();
        let buffer = &((*b).borrow().buffer);
        let items = page_info.item_map;
        for item in items.iter()  {
          if item.uuid == uuid {
              let offset = item.offset as usize;
              let size = item.size as usize;
              let start = offset - size;
              let mut v = Vec::new();
              for index in start..offset {
                  v.push(buffer[index]);
              }
              return Option::Some(v);
          }
        };
        None
    }

    pub fn update_item_by_uuid(&mut self,uuid:u32,new_item:Vec<u8>){
        let page_id = (uuid >> 16);
        let p_id = page_id + 1 ;
        let b:&RefCell<BufferPool> = self.buffer_pool.borrow();
        let mut buffer_pool = b.borrow_mut();
        let page = buffer_pool.get_page_lru(self.file_name.as_str(), p_id);
        let page_info = self.judge_page(&page).unwrap();
        let b = page.buffer.clone();
        let mut buffer = &mut (*page.buffer).borrow_mut().buffer;
        let items = page_info.item_map;

        for item in items {
            if item.uuid == uuid {
                let offset = item.offset as usize;
                let size = item.size as usize;
                let start = offset - size;
                if size != new_item.len() {
                    panic!("更新长度需要固定");
                }else {
                    for i in 0..size as usize {
                        buffer[i+start]  = new_item[i];
                    }
                }
            }
        }

        panic!("没有对应的数据");
    }
}