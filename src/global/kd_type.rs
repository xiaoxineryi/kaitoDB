use super::global;
use std::sync::Once;
use crate::global::global::{Global, GLOBAL};
use std::path::Path;
use std::fs::File;
use std::any::TypeId;
use std::io::{Write, Read};
use std::borrow::Borrow;
use std::convert::TryInto;

type OID = u32;

#[derive(Debug)]
pub struct kd_type{
    typ_name:String,
    typ_oid:OID,
    typ_type:char,
    typ_len:u8,
}
static INIT: Once = Once::new();
pub static mut TYPES: Vec<kd_type> = Vec::new();

impl kd_type{
    pub fn init_types(){
        INIT.call_once(|| unsafe {
           kd_type::init_types_privacy(Global::get_path());
        });
    }

    unsafe fn init_types_if_none(file:File){
        TYPES.push(kd_type{
            typ_name: "u32".to_string(),
            typ_oid: Global::get_and_add_oid(),
            typ_type: 'u',
            typ_len: 4
        });
        TYPES.push(kd_type{
            typ_name: "i32".to_string(),
            typ_oid: Global::get_and_add_oid(),
            typ_type: 'i',
            typ_len: 4
        });
        TYPES.push(kd_type{
            typ_name: "float".to_string(),
            typ_oid: Global::get_and_add_oid(),
            typ_type: 'f',
            typ_len: 4
        });
        TYPES.push(kd_type{
            typ_name: "char".to_string(),
            typ_oid: Global::get_and_add_oid(),
            typ_type: 'c',
            typ_len: 0
        });
        TYPES.push(kd_type{
            typ_name: "varchar".to_string(),
            typ_oid: Global::get_and_add_oid(),
            typ_type: 'v',
            typ_len: 0
        });
        kd_type::save_types(file);
    }
    unsafe fn save_types(mut file :File){
        for k_type in TYPES.iter() {
            println!("{:?}",k_type);
            //先存oid、类型、长度的信息，最后因为string类型可变，先存长度，然后用长度读取
            file.write(k_type.typ_oid.to_be_bytes().borrow()).unwrap();
            file.write(&[u8::from_be(k_type.typ_type as u8)]).unwrap();
            file.write(&[k_type.typ_len]).unwrap();
            file.write(&[k_type.typ_name.len() as u8]);
            file.write(k_type.typ_name.as_bytes());
        }
    }

    unsafe fn load_types(mut file:File){
        let mut off_set = 0usize;
        let mut buffer :[u8;512] = [0;512];
        let size = file.read(&mut buffer).unwrap();
        for index in 0..Global::get_type_number() {
            let temp = &buffer[off_set..off_set+4];
            let oid = u32::from_be_bytes(temp.try_into().unwrap());
            off_set += 4;
            let k_type = buffer[off_set];
            off_set += 1;
            let k_len = buffer[off_set];
            off_set += 1;
            let name_len = buffer[off_set];
            off_set += 1;
            let name = &buffer[off_set..off_set + name_len as usize];
            let name = String::from_utf8_lossy(name);
            off_set += name_len as usize;

            TYPES.push(kd_type{
                typ_name: name.to_string(),
                typ_oid: oid,
                typ_type: k_type as char,
                typ_len: k_len
            });

            println!("{:?}",TYPES[index as usize]);
        }


    }

    fn init_types_privacy(path:String){
        let file_path = path+"/types.setting";
        let file_path = Path::new(file_path.as_str());
        match File::open(file_path) {
            Ok(_)=>{println!("文件{}已找到，正在读取基本数据类型",file_path.display());},
            Err(_)=>{
                println!("文件{}不存在,正在初始化基本数据类型",file_path.display());
                match File::create(file_path) {
                    Ok(file)=> unsafe {
                        kd_type::init_types_if_none(file);
                    },
                    Err(err)=>{panic!("初始化基本数据类型错误，错误类型为{}",err);}
                }
            }
        }

        let file = match File::open(file_path) {
            Ok(file)=> file,
            Err(err)=>{panic!("文件无法打开,错误类型为{}",err);}
        };

        unsafe { kd_type::load_types(file); }
    }

}