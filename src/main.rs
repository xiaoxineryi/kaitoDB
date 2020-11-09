
mod global;
use global::global::Global;
use global::kd_type::kd_type;
use global::kd_database::kd_database;



fn main() {
    unsafe {
        Global::init_global(None);
        Global::debug();
        kd_type::init_types();
        kd_database::init_db();
    }
}
