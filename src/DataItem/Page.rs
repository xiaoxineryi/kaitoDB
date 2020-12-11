
use crate::SIZE;
use crate::Disk::Disk::DiskManager;
use std::convert::TryInto;

struct PageHeader{
    is_page:u32,
    page_number:u32,
    old_lsn:u64,
    new_lsn:u64,
    record_number:u16,
    free_size:u16,
    is_free:bool
}

struct ItemInfo{
    offset:u16,
    OOID:u32,
    size:u16
}

pub struct Page{
    page_header:PageHeader,
    item_map:Vec<ItemInfo>,

}

pub struct ItemHandler{
    pub(crate) file_name:String,
    pub(crate) free_page:u32
}
pub struct ItemManager{}
impl ItemManager{
    pub fn new_item_handler(file_name:String,size:u32)->ItemHandler{
        ItemHandler{
            file_name,
            free_page: size
        }
    }
}
impl ItemHandler{
    // 直接获取一页数据
    fn get_items_by_page(&self,page_id:u32){

    }
    // 获取某个数据
    fn get_item_by_uuid(&self,uuid:u32){

    }
}