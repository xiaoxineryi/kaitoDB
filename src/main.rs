
mod pg;
mod global;

use pg::pg_setting::pg_database as pg_database;



fn main() {
    let a = pg_database::new();
    a.print();
    unsafe {
        global::global::Global::init_global(None);
        let id = global::global::Global::get_and_add_oid();
        global::global::Global::debug();
        global::kd_type::kd_type::init_types();
    }
}
