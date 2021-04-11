
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{Read, Write, Seek, SeekFrom};
use rand::AsByteSliceMut;
use std::convert::TryInto;
use crate::SIZE;


//
// static m:HashMap<String,u8> =HashMap::new() ;

pub struct DiskHeader{
    pub page_number:u32
}
pub struct DiskHandler{
    pub disk_header:DiskHeader,
    pub file_name:String
}

pub struct DiskManager{}
impl DiskManager{
   pub fn create_file(file_name:&str, page_size:u32) ->DiskHandler{
        let mut file = match File::open(String::from("./")+file_name) {
            Ok(_)=>{
                panic!("文件名已存在");
            }
            Err(_)=>{
                OpenOptions::new().write(true).create(true).append(true).read(true).open(String::from("./")+file_name).unwrap()
            }
        };
        let mut buffer = [0u32;1024];
        buffer[0] = page_size;
       println!("page size is {}",page_size);
        file.write(buffer.as_byte_slice_mut()).unwrap();
        buffer[0] = 0;
        for index in 1..page_size  {
            file.write(buffer.as_byte_slice_mut()).unwrap();
        };
        let disk_header = DiskHeader{ page_number: page_size };
        DiskHandler{
            disk_header,
            file_name:String::from(file_name)
        }
    }
    pub fn get_file(file_name:&str)->DiskHandler{
        let mut file  = fs::File::open(String::from("./")+file_name).unwrap();
        let mut buffer = [0u8;4096];
        file.read(&mut buffer).unwrap();
        let b = &buffer[0..4];
        // println!("{:?}",b);

        let size = u32::from_le_bytes(b.try_into().unwrap());
        let disk_header = DiskHeader{ page_number: size };
        DiskHandler{
            disk_header,
            file_name:String::from(file_name)
        }
    }
}

impl DiskHandler{
    pub fn get_page(&self, pageID:u32)->[u8;SIZE]{
        let mut file = OpenOptions::new().read(true).open(String::from("./")+&self.file_name).unwrap();
        file.seek(SeekFrom::Start((((pageID+1) * SIZE as u32) as u64)));
        let mut buffer:[u8;SIZE] = [0u8;SIZE];
        file.read(&mut buffer).unwrap();
        buffer
    }

    pub fn flush_page(&self,pageID:u32,buffer:[u8;SIZE]){
        let mut file = OpenOptions::new().write(true).open(String::from("./")+&self.file_name).unwrap();
        file.seek(SeekFrom::Start(((pageID+1) * SIZE as u32) as u64));
        file.write(&buffer).unwrap();
    }
}



