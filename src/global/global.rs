use std::sync::Once;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{ Seek, SeekFrom};
use std::path::Path;
use std::result::Result::Err;
use std::fs;


use std::convert::TryInto;
use std::intrinsics::transmute;
use std::borrow::Borrow;

type OID = u32;

#[derive(Debug)]
pub struct Global{
    oid:OID,
    db_number : OID,
    path: String,
    type_number:u8
}

pub static mut GLOBAL:Option<Global> = None;
static INIT: Once = Once::new();
static DEFAULT_PATH : &str= "/home/xiawenke/kaitoDB";

impl Global{
    fn init_global_from_file(dir:String){
        fs::create_dir_all(dir.clone());
        let d  =dir.clone();
        let file_name = d+ "/global.setting";
        let file_name = Path::new(file_name.as_str());
        println!("正在寻找对应的初始化文件:{}",file_name.display());


        match File::open(file_name) {
            Ok(_) =>{}
            Err(_)=>{
                println!("数据库文件{}不存在,正在新建数据库.", file_name.display());
                match File::create(file_name){
                    Ok(file)=>  {
                        Global::init_database_if_none(file,dir.clone());
                    }
                    Err(_) =>{
                        panic!("无法新建数据库");
                    }
                }
            }
        };
        let  file = match File::open(file_name) {
            Ok(file)=>file,
            Err(_)=>{panic!("文件{}无法打开",file_name.display())}
        };
        println!("{:?}",file);
        Global::load_data(file);
    }


    fn load_data(mut file:File){
        let mut buffer :[u8;512] = [0;512];
        let size = file.read(&mut buffer).unwrap();
        let temp = &buffer[0..4];
        let oid = u32::from_be_bytes(temp.try_into().unwrap());
        let db_temp = &buffer[4..8];
        let db_number= u32::from_be_bytes(db_temp.try_into().unwrap());
        let type_number = buffer[8];
        let src = &buffer[9..size];
        println!("{}",size);
        let s = String::from_utf8_lossy(src);
        println!("{}",s);
        unsafe {
            GLOBAL = Option::from(Global {
                oid: oid ,
                db_number: db_number,
                path: s.to_string(),
                type_number: type_number
            });
        }
    }

    fn init_database_if_none(mut file:File,dir:String)->File{
        file.write(&[0u8,0u8,0u8,0u8]).unwrap();
        file.write(&[0u8,0u8,0u8,0u8]).unwrap();
        file.write(&[5u8]).unwrap();
        file.write(dir.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    pub unsafe fn get_db_number()->OID{
        GLOBAL.as_mut().unwrap_or_else(||panic!("全局数据库信息未初始化")).db_number
    }

    pub unsafe fn update_db_number(oid:OID){
        GLOBAL.as_mut().unwrap().db_number = oid;
        let g  = GLOBAL.as_mut().unwrap_or_else(||panic!("全局数据库信息未初始化"));
        let mut file  = OpenOptions::new()
            .write(true).open(Path::new((g.path.clone()+"/global.setting").as_str())).unwrap();
        file.seek(SeekFrom::Start(4)).unwrap();
        file.write(oid.to_be_bytes().borrow()).unwrap();
    }

    pub unsafe fn get_type_number()->u8{
        GLOBAL.as_mut().unwrap_or_else(||panic!("全局数据库信息未初始化")).type_number
    }

    pub unsafe fn get_path() ->String{
        GLOBAL.as_mut().unwrap_or_else(||panic!("全局数据库信息未初始化")).path.clone()
    }

    pub unsafe fn init_global(val:Option<String>){
        let path = if val.is_none() {String::from(DEFAULT_PATH)}else{val.unwrap()};
        INIT.call_once(||{
            Global::init_global_from_file(path);
        });
    }

    pub unsafe fn get_and_add_oid() ->OID{
        let g  = GLOBAL.as_mut().unwrap_or_else(||panic!("全局数据库信息未初始化"));
        g.oid +=1;
        let mut file  = OpenOptions::new()
            .write(true).open(Path::new((g.path.clone()+"/global.setting").as_str())).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write(g.oid.to_be_bytes().borrow()).unwrap();
        let id = g.oid;
        id
    }
    pub unsafe fn debug(){
        println!("{:?}",GLOBAL);
    }
}

/*

可以使用trait+函数添加不同的参数，来实现函数的重载。
pub trait INIT<T>{
    unsafe fn init_global(val:T);
}
impl INIT<i8> for Global{
    unsafe fn init_global(val: i8) {
        Global::init_global(String::from(DEFAULT_PATH))
    }
}

impl INIT<String> for Global{
    unsafe fn init_global(val: String){
        GLOBAL = Option::from(Global {
            oid: 0,
            db_number: 0,
            path: val
        });
    }
}
 */
