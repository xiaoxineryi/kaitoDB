use super::kd_table::kd_table;
use std::sync::Once;
use super::global::Global;
use crate::global::global::GLOBAL;
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::io::{Write, Read};
use std::borrow::Borrow;
use std::convert::TryInto;

type OID = u32;

pub struct kd_database{
    db_name:String,
    db_id:OID,
    table_number:u8,
    table_ids:Vec<OID>,
    table_names:Vec<String>,
    tables:Vec<kd_table>
}
static INIT: Once = Once::new();
pub static mut DATABASE:Option<kd_database> = None;

impl kd_database{
    pub unsafe fn init_db(){
        INIT.call_once(||{
            kd_database::init_db_privacy();
        });
    }

    unsafe fn init_db_privacy(){
        let db_oid = Global::get_db_number();
        if db_oid == 0 {kd_database::create_db();}
        kd_database::load_db();
    }

    unsafe fn create_db(){
        println!("路径中没有找到数据库信息，正在建立");
        let mut name = String::new();
        println!("请输入您要创建数据库的名字");
        let name_size = std::io::stdin().read_line(&mut name).unwrap();
        let db_oid = Global::get_and_add_oid();
        Global::update_db_number(db_oid);
        println!("the db_number is {}",Global::get_db_number());
        let dir_path = kd_database::getPath(db_oid);
        create_dir_all(dir_path.as_str());
        let file_path = dir_path + "/db.setting";
        let path = Path::new(file_path.as_str());
        match File::create(path){
            Ok(file)=>{kd_database::init_file(file,name,db_oid)},
            Err(err)=>{panic!("新建文件{}失败，失败原因为{}",path.display(),err);}
        };
    }

    fn init_file(mut file:File, db_name:String, db_oid:OID){
        // 写入顺序为 数据库名称字节数、数据库名、数据库编号、存放表大小
        file.write(&[db_name.len() as u8]).unwrap();
        file.write(db_name.as_bytes()).unwrap();
        file.write(db_oid.to_be_bytes().borrow()).unwrap();
        file.write(&[0u8]).unwrap();
    }

    unsafe fn getPath(oid:OID) -> String{
        let mut path = Global::get_path();
        path +"/"+ oid.to_string().as_str()
    }

    unsafe fn load_db(){
        let db_oid = Global::get_db_number();
        let path = kd_database::getPath(db_oid)+"/db.setting";
        let mut file = match File::open(Path::new(path.as_str())){
            Ok(file)=> file,
            Err(err)=>{panic!("读取数据库信息{}时出现问题,错误类型为{}",path,err);}
        };
        let mut buffer:[u8;512] = [0;512];
        let size = file.read(&mut buffer).unwrap();
        let mut off_set = 0;
        // 读取数据库名的字节数
        let name_size = buffer[off_set];
        off_set += 1;
        // 读取数据库名
        let db_name = &buffer[off_set..off_set+name_size as usize];
        off_set += name_size as usize;
        // 读取oid
        let temp = &buffer[off_set..off_set + 4];
        let db_oid = u32::from_be_bytes(temp.try_into().unwrap());
        off_set += 4;
        //读取表个数
        let table_number = buffer[off_set];
        off_set += 1;
        DATABASE = Option::from(kd_database {
            db_name:String::from_utf8_lossy(db_name).to_string(),
            db_id: db_oid,
            table_number:table_number,
            table_ids: vec![],
            table_names: vec![],
            tables: vec![]
        });
        //读取所有的表的名字和OID
        for index in 0..table_number {
            let temp = &buffer[off_set..off_set + 4];
            let table_oid = u32::from_be_bytes(temp.try_into().unwrap());
            off_set += 4;

            let table_name_size = buffer[off_set];
            off_set += 1;
            // 读取数据库名
            let table_name = &buffer[off_set..off_set+table_name_size as usize];
            off_set += table_name_size as usize;
            DATABASE.as_mut().unwrap().table_ids.push(table_oid);
            DATABASE.as_mut().unwrap().table_names.push(
                String::from_utf8_lossy(table_name).to_string());
        }

    }

}