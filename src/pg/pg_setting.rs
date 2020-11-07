#[derive(Debug)]
pub struct pg_database{
    pub oid:i32,
    pub database_name:String
}

impl pg_database{
    pub fn print(&self){
        println!("{:?}",self);
    }
    pub fn new()->pg_database{
        pg_database{ oid: 0, database_name: "你好".to_string() }
    }
}
