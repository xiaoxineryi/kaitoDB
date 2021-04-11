use crate::SIZE;
use crate::Disk::Disk::DiskManager;
use crate::DataItem::Page::{ItemHandler, ItemManager};
use std::borrow::Borrow;
use std::convert::TryInto;
use std::rc::Rc;
use crate::BufferPool::BufferPool::BufferPool;
use std::cell::{Ref, RefCell};

#[derive(Clone)]
pub struct Format{
   pub attr_name:String,
   pub attr_type:u8
}



pub struct TableManager{
   pub(crate) buffer_pool:Rc<RefCell<BufferPool>>
}

impl TableManager{
   fn save_attr(page:&mut[u8;SIZE],attrs:Vec<Format>,s:usize){
      let mut start = s;
      for attr in attrs.iter() {
         // 存储属性类型
         page[start] = attr.attr_type;
         start += 1;
         // 存储属性名称大小
         page[start] = attr.attr_name.len() as u8;
         start += 1;
         // 存储属性名称
         let name = attr.attr_name.as_bytes();
         for index in 0..name.len() {
            page[start] = name[index];
            start += 1;
         }
      }
   }
   fn save_free(page:&mut[u8;SIZE],free_page:u32){
      let e = free_page.to_be_bytes();
      for index in 0..4 as usize {
         page[index] = e[index];
      }
   }
   pub fn create_table(&self,file_name:&str,attrs:Vec<Format>,size:u32)->TableHandler{
      let disk_handler = DiskManager::create_file(file_name,size);
      let mut page = disk_handler.get_page(0);
      let c = attrs.clone();
      // 赋值第一个空闲的页
      TableManager::save_free(&mut page,0u32);
      //存储属性个数
      let attr_num = attrs.len() as u8;
      page[4] = attr_num;
      //存储各项属性
      TableManager::save_attr(&mut page,attrs,5);
      disk_handler.flush_page(0,page);
      TableHandler{
         file_name: String::from(file_name),
         attr_num,
         attr_format: c.clone(),
         page_handler: ItemManager::new_item_handler(String::from(file_name),0,self.buffer_pool.clone())
      }
   }

   pub fn open_table(&self,file_name:&str)->TableHandler{
      let disk_handler = DiskManager::get_file(file_name);
      let page = disk_handler.get_page(0);
      TableHandler::get_table_handler(file_name,&page,self.buffer_pool.clone())
   }
}

pub struct TableHandler{
   file_name:String,
   pub attr_num:u8,
   pub attr_format:Vec<Format>,
   pub page_handler:ItemHandler
}

impl TableHandler {
   fn get_table_handler(file_name:&str,page:&[u8;SIZE],buffer_pool:Rc<RefCell<BufferPool>>)->TableHandler{
      let free_id = u32::from_be_bytes((&page[0..4]).try_into().unwrap());
      let attr_num = page[4];
      let mut formats:Vec<Format> = Vec::new();
      let mut start = 5;
      for index in 0..attr_num as u8 {
         let attr_type = page[start];
         start += 1;
         let attr_size = page[start];
         start += 1;
         let mut name = vec![];
         for i in 0..attr_size{
            name.push(page[start]);
            start += 1;
         }
         let attr_name = String::from_utf8_lossy(name.as_slice());
         formats.push(Format{
            attr_name:attr_name.to_string(),
            attr_type
         });
      };
      TableHandler{
         file_name: String::from(file_name),
         attr_num,
         attr_format: formats,
         page_handler: ItemManager::new_item_handler(String::from(file_name),free_id,buffer_pool.clone())
      }
   }


   // 解析数据
   pub fn parse_item(&self,item:&Vec<u8>){
      let mut start = 0usize;
      for f in self.attr_format.iter() {
         let size = item[start];
         let attr = &item[start..start+1+size as usize];
         println!("属性名为:{}",f.attr_name);
         self.parse_attr(attr,f.attr_type);
         start += 1+size as usize;
      }
   }
   // 解析属性
   fn parse_attr(&self,attr:&[u8],attr_type:u8){
      let size = attr[0] as usize;
      match attr_type {
         1=>{
            // 如果是1表示是u32类型的
            let e = u32::from_be_bytes(attr[1..1+size].try_into().unwrap());
            println!("{}",e);
         },
         2=>{
            //  如果是2的话 表示是varchar类型的
            let e = String::from_utf8_lossy(attr[1..1+size].try_into().unwrap());
            println!("{}",e);
         },
         3=>{
            println!("我是float");

         },
         _=>{
           panic!("没有对应的属性信息")
         },
      }
   }
}