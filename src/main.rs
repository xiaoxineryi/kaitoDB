use crate::Disk::Disk::DiskManager;
use std::io::{Read, Write};
use std::borrow::Borrow;
use std::rc::Rc;
use std::ops::Deref;

mod DataItem;
mod Disk;
mod BufferPool;
mod Record;
mod Test;

const SIZE:usize = 4096;

fn main(){


    let mut bufferPool = BufferPool::BufferPool::BufferPool::default();
    let p = bufferPool.get_page("1.txt",0);
    let s =p.buffer.clone();
    {
        let mut e = s.borrow_mut();
        for i in e.buffer.iter() {
            print!("{}",i);
        }
    }
    {
        let mut e = s.borrow_mut();
        e.is_dirty = true;
    }
    bufferPool.flush_page(0);
    println!("{}",Rc::strong_count(&s));

}


