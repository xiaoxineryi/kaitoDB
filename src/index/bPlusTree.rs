use std::sync::{Arc, RwLock};
use crate::index::node::{Node, NodeType, INDEX_RECORD_SIZE, IndexRecord};
use crate::DataItem::Page::{ItemHandler, ItemManager};
use crate::Disk::Disk::DiskManager;
use std::rc::Rc;
use std::cell::RefCell;
use crate::BufferPool::BufferPool::BufferPool;
use std::convert::TryInto;
use crate::index::key_value_pair::KeyValuePair;
use crate::index::error::Error;

const M:u32 = 5;

pub struct BPlusTree {
    pub root: Arc<RwLock<Node>>,
    free_page:u32,
    item_handler: ItemHandler,
}

impl BPlusTree {
    pub fn create_index(file_name :&str,size:u32,buffer_pool:Rc<RefCell<BufferPool>>)-> BPlusTree {
        let disk_handler = DiskManager::create_file(file_name,size);
        let mut page = disk_handler.get_page(0);
        let mut item_handler = ItemManager::new_item_handler(String::from(file_name), 0, buffer_pool);
        let mut root = Vec::<u8>::new();
        //构建一个空的根
        // 添加表示是根
        root.push(0x01);
        // 记录父节点uuid
        for i in 0..INDEX_RECORD_SIZE-1 {
            root.push(0);
        }
        // 记录根节点
        let uuid = item_handler.insert_item(root.clone());
        let uuid_raw = uuid.to_be_bytes();
        //创建一个索引表，在第一页放入所要加入的数据，这里为根节点的uuid。
        for i in 0..uuid_raw.len() {
            page[i] = uuid_raw[i];
        }
        //同时记录空闲页
        let free_page_raw = 0u32.to_be_bytes();
        for i in 0..free_page_raw.len() {
            page[i + 4] = free_page_raw[i];
        }
        disk_handler.flush_page(0,page);
        let node = Node{
            node_type: NodeType::Root,
            parent_uuid: 0,
            uuid,
            content: IndexRecord{ data: root.clone() }
        };
        BPlusTree { root: Arc::new((RwLock::new(node))), free_page: 0, item_handler }
    }

    pub fn open_index(file_name:&str,buffer_pool:Rc<RefCell<BufferPool>>)-> BPlusTree {
        let disk_handler = DiskManager::get_file(file_name);
        let mut page = disk_handler.get_page(0);
        let mut item_handler = ItemManager::new_item_handler(String::from(file_name), 0, buffer_pool);
        let root_uuid = u32::from_be_bytes(page[0..4].try_into().unwrap());
        let free_page = u32::from_be_bytes(page[4..8].try_into().unwrap());
        let root = item_handler.get_item_by_uuid(root_uuid).unwrap();
        let node = Node{
            node_type: NodeType::Root,
            parent_uuid: 0,
            uuid: root_uuid,
            content: IndexRecord { data: root.clone() }
        };
        BPlusTree { root: Arc::new(RwLock::new(node)), free_page, item_handler }
    }

    pub fn search(&mut self, key: u32) -> Result<KeyValuePair, Error> {
        let (_, kv) = self.search_node(Arc::clone(&self.root), key)?;
        match kv {
            Some(kv) => return Ok(kv),
            None => return Err(Error::KeyNotFound),
        }
    }

    pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        let (node, kv_pair_exists) = self.search_node(Arc::clone(&self.root), kv.key)?;
        match kv_pair_exists {
            // 如果键值对已经存在，就报错，不支持相同键多次插入
            Some(_) => return Err(Error::KeyAlreadyExists),
            None => (),
        };
        // 获取到对应要添加位置的叶子节点
        let mut guarded_node = match node.write() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        // 进行判断大小够不够
        let key_number = guarded_node.get_keys_number()?;
        if key_number < key_number {
            // 如果可以直接插入的话，就直接插入
            guarded_node.add_key_value_pair(kv)?;
            return Ok(())
        }else {
            self.split_node(Arc::clone(&node))?;
            Ok(())
        }

    }
    pub fn insert_internal(&mut self,node : Arc<RwLock<Node>>,kv: KeyValuePair) -> Result<(), Error> {
        let (node, kv_pair_exists) = self.search_node(node, kv.key)?;
        match kv_pair_exists {
            // 如果键值对已经存在，就报错，不支持相同键多次插入
            Some(_) => return Err(Error::KeyAlreadyExists),
            None => (),
        };
        // 获取到对应要添加位置的叶子节点
        let mut guarded_node = match node.write() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        // 进行判断大小够不够
        let key_number = guarded_node.get_keys_number()?;
        if key_number < key_number {
            // 如果可以直接插入的话，就直接插入
            guarded_node.add_key_value_pair(kv)?;
            return Ok(())
        }else {
            self.split_node(Arc::clone(&node))?;
            Ok(())
        }

    }


    fn split_node(&mut self, node: Arc<RwLock<Node>>) -> Result<(), Error> {
        let mut guarded_node = match node.write() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        let keys = guarded_node.get_keys()?;
        let median_key = keys[keys.len() / 2];
        // 获取父亲节点相应内容
        let parent_uuid = guarded_node.get_parent_uuid()?;
        let parent_buffer = self.item_handler.get_item_by_uuid(parent_uuid).unwrap();
        let parent_node = Node::create_from(parent_buffer,parent_uuid);
        //分裂子节点
        let node_add = guarded_node.split().unwrap();

        self.insert_internal(Arc::new(RwLock::new(parent_node)),KeyValuePair{ key: median_key, value: 0 })
    }


    fn search_node(
        &mut self,
        node: Arc<RwLock<Node>>,
        search_key: u32,
    ) -> Result<(Arc<RwLock<Node>>, Option<KeyValuePair>), Error> {
        let guarded_node = match node.read() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        let keys = guarded_node.get_keys()?;
        for (i, key) in keys.iter().enumerate() {
            // If this is the case were at a leaf node.
            if *key == search_key && node.read().unwrap().node_type == NodeType::Leaf{
                let kv_pairs = guarded_node.get_key_value_pairs()?;
                match kv_pairs.get(i) {
                    None => return Err(Error::UnexpectedError),
                    Some(kv) => return Ok((Arc::clone(&node), Some(kv.clone()))),
                };
            }
            if *key >= search_key {
                return self.traverse_or_return(Arc::clone(&node), i, search_key);
            }
        }
        self.traverse_or_return(Arc::clone(&node), keys.len(), search_key)
    }

    fn traverse_or_return(
        &mut self,
        node: Arc<RwLock<Node>>,
        index: usize,
        search_key: u32,
    ) -> Result<(Arc<RwLock<Node>>, Option<KeyValuePair>), Error> {
        let guarded_node = match node.read() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        match guarded_node.node_type {
            NodeType::Leaf => return Ok((Arc::clone(&node), None)),
            NodeType::Internal | NodeType::Root => {
                let children_uuids = guarded_node.get_children()?;
                let child_uuid = match children_uuids.get(index) {
                    None => return Err(Error::UnexpectedError),
                    Some(child_uuid) => child_uuid,
                };
                let child_node = self.get_node_by_uuid(*child_uuid as u32)?;
                return self.search_node(Arc::new(RwLock::new(child_node)), search_key);
            }
            NodeType::Unknown => return Err(Error::UnexpectedError),
        };
    }

    fn get_node_by_uuid(&mut self,uuid:u32)->Result<Node,Error>{
        let item = self.item_handler.get_item_by_uuid(uuid).unwrap();
        Ok(Node::create_from(item,uuid))
    }


    pub fn delete_index(&mut self,search_key:u32){
        let (node,key_value) = self.search_node(self.root.clone(),search_key).unwrap();
        let uuid = node.read().unwrap().uuid;
        let mut record = self.item_handler.get_item_by_uuid(key_value.unwrap().value).unwrap();
        let mut node = Node::create_from(record, uuid);
        node.delete_record(search_key);
        self.item_handler.update_item_by_uuid(uuid,node.content.data);
        println!("数据已删除");
    }



}