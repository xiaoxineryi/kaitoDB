
mod pg;
mod global;

use pg::pg_setting::pg_database as pg_database;
use crate::global::Global;


fn main() {
    let a = pg_database::new();
    a.print();
    unsafe {
        global::Global::init_global(None);
        let id = Global::get_and_add_oid();
        Global::print();
    }

}
