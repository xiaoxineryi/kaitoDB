use crate::index::key_value_pair::KeyValuePair;
use crate::index::error::Error;
use std::convert::{TryFrom, TryInto};
use std::borrow::Borrow;

//先规定大小和位置
// 节点类型的大小
const NODE_TYPE_SIZE : usize  = 1 ;
// 父节点uuid大小
const PARENT_UUID_SIZE : usize = 4;
//有多少个记录的大小
const INDEX_NUMBER_SIZE : usize = 1;
//头信息大小
const HEADER_SIZE :usize = NODE_TYPE_SIZE + PARENT_UUID_SIZE + INDEX_NUMBER_SIZE;
// 记录的左子树的uuid大小
const LEFT_NODE_UUID_SIZE :usize = 4;
// 主键的大小
const KEY_SIZE :usize = 4;
// 数据的大小
const VALUE_SIZE :usize = 4;
// 是否被删除
const IS_DELETE_SIZE :usize = 1;
//一个index的大小
const INDEX_INFO_SIZE:usize = LEFT_NODE_UUID_SIZE + KEY_SIZE + VALUE_SIZE + IS_DELETE_SIZE;
//右子树大小
const RIGHT_NODE_UUID_SIZE :usize = 4;
//最多一个节点中有多少数据
const INDEX_MAX_NUMBER :usize = 5;

pub const INDEX_RECORD_SIZE :usize = HEADER_SIZE + INDEX_MAX_NUMBER * INDEX_INFO_SIZE + RIGHT_NODE_UUID_SIZE;

const PTR_SIZE :usize = 4;

//要获取节点的记录个数需要的偏移量
const INDEX_NUMBER_OFFSET:usize = NODE_TYPE_SIZE + PARENT_UUID_SIZE;
const INDEX_START_OFFSET :usize = HEADER_SIZE;
#[derive(PartialEq)]
pub enum NodeType{
    Internal = 1,
    Leaf = 2,
    Root = 3,
    Unknown
}


// Casts a byte to a NodeType.
impl From<u8> for NodeType {
    fn from(orig: u8) -> Self {
        match orig {
            0x01 => return NodeType::Internal,
            0x02 => return NodeType::Leaf,
            0x03 => return NodeType::Root,
            _ => return NodeType::Unknown,
        };
    }
}

trait FromByte {
    fn from_byte(&self) -> bool;
}

trait ToByte {
    fn to_byte(&self) -> u8;
}

impl FromByte for u8 {
    fn from_byte(&self) -> bool {
        match self {
            0x01 => return true,
            _ => return false,
        };
    }
}

impl ToByte for bool {
    fn to_byte(&self) -> u8 {
        return match self {
            true => 0x01,
            false => 0x00,
        };
    }
}

pub struct  IndexRecord{
    pub data:Vec<u8>
}

impl IndexRecord{
    pub fn get_value_from_offset(&self, offset: usize) -> Result<u32, Error> {
        let bytes = &self.data[offset..offset + PTR_SIZE];
        let res = u32::from_be_bytes(bytes.try_into().unwrap());
        Ok(res)
    }
}

// B+树的节点
pub struct Node{
    pub node_type : NodeType,
    pub parent_uuid : u32,
    pub uuid : u32,
    pub content:IndexRecord
}

impl Node{

    pub fn split(&mut self)->Result<Node,Error>{
        let mid_offset = HEADER_SIZE + INDEX_INFO_SIZE * 2;
        let v = self.content.data.clone();
        let parent_uuid = u32::from_be_bytes(v[1..5].try_into().unwrap());
        let mut vec = Vec::<u8>::new();
        vec.push(0x01);
        let parent_uuid_raw = parent_uuid.to_be_bytes();
        for index in parent_uuid_raw.iter() {
            vec.push(*index);
        };
        for i in mid_offset..INDEX_RECORD_SIZE {
            vec.push(self.content.data[i]);
        };
        Ok(Node::create_from(vec,parent_uuid))
    }

    pub fn get_parent_uuid(&mut self)->Result<u32,Error>{
        let v = &self.content.data;
        let parent_uuid = u32::from_be_bytes(v[1..5].try_into().unwrap());
        Ok(parent_uuid)
    }

    pub fn delete_record(&mut self, delete_key:u32){
        let keys = self.get_keys().unwrap();
        let mut offset = INDEX_START_OFFSET;
        for (index,key) in keys.iter().enumerate()  {
            if *key == delete_key {
                offset += INDEX_INFO_SIZE - IS_DELETE_SIZE;
                self.content.data[offset] = 0x01;
            }
            offset += INDEX_INFO_SIZE;
        }
    }

    pub fn get_keys_number(&self) -> Result<u32, Error> {
        match self.node_type {
            NodeType::Internal | NodeType::Root | NodeType::Leaf => {
                let num_children = self
                    .content
                    .get_value_from_offset(INDEX_NUMBER_OFFSET)?;

                return Ok(num_children);
            }
            _ => return Err(Error::UnexpectedError),
        };
    }

    pub fn add_key_value_pair(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        match self.node_type {
            NodeType::Leaf => {
                let num_keys_val_pairs = self
                    .content
                    .get_value_from_offset(INDEX_NUMBER_OFFSET)?;
                let mut offset = HEADER_SIZE + LEFT_NODE_UUID_SIZE;
                for i in 1..=num_keys_val_pairs as usize {
                    let key = self.content.get_value_from_offset(offset)?;
                    if key > kv.key {
                        //数据全都往后移动一个数据项
                        for index in offset-LEFT_NODE_UUID_SIZE .. INDEX_RECORD_SIZE - INDEX_INFO_SIZE {
                            self.content.data[index + INDEX_INFO_SIZE] = self.content.data[index];
                        }
                        let key_raw = kv.key.to_be_bytes();
                        for key_index in 0..key_raw.len() {
                            self.content.data[offset ] = key_raw[key_index];
                            offset += 1;
                        }
                        let value_raw = kv.value.to_be_bytes();
                        for value_index in 0..value_raw.len() {
                            self.content.data[offset] = value_raw[value_index];
                            offset += 1;
                        }

                        break;
                    }
                }

                Ok(())
            }
            _ => return Err(Error::UnexpectedError),
        }
    }

    pub fn create_from(v:Vec<u8>,uuid:u32)->Node{
        let node_type = v.get(0).unwrap();
        let t = NodeType::from(*node_type);

        let parent_uuid = u32::from_be_bytes(v[1..5].try_into().unwrap());
        Node{
            node_type: t,
            parent_uuid,
            uuid,
            content: IndexRecord { data: v }
        }
    }
    pub fn new(
        node_type:NodeType,
        parent_uuid : u32,
        is_root : bool,
        uuid:u32,
        content:Vec<u8>
    ) ->Node{
        Node{
            node_type,
            parent_uuid,
            uuid,

            content: IndexRecord {
                data: content
            }
        }
    }

    pub fn get_key_value_pairs(&self )->Result<Vec<KeyValuePair>,Error>{
        match self.node_type {
            NodeType::Leaf =>{

                let mut res  = Vec::<KeyValuePair>::new();
                let mut offset = INDEX_NUMBER_OFFSET;
                let num_keys_val_pairs = self.content.get_value_from_offset(offset)?;

                offset = INDEX_START_OFFSET + LEFT_NODE_UUID_SIZE;
                for _i in 0.. num_keys_val_pairs{
                    let key = self.content.get_value_from_offset(offset)?;
                    offset += KEY_SIZE;

                    let value = self.content.get_value_from_offset(offset)?;
                    offset += VALUE_SIZE;

                    let is_delete = u8::from_byte(&self.content.data[offset]);
                    offset += IS_DELETE_SIZE;
                    offset += LEFT_NODE_UUID_SIZE;
                    if is_delete {
                        continue
                    }
                    res.push(KeyValuePair::new(
                        key,
                        value
                    ));

                }
                return Ok(res);
            }
            _ => {
                return Err(Error::UnexpectedError)
            }
        };
    }


    // 因为是统一进行的格式处理，所以不管是哪种类型数据都是一样的解析方法
    pub fn get_keys(&self) -> Result<Vec<u32>, Error> {
        match self.node_type {
            NodeType::Internal | NodeType::Root | NodeType::Leaf => {
                let mut res = Vec::<u32>::new();
                let num_children = self
                    .content
                    .get_value_from_offset(INDEX_NUMBER_OFFSET)?;

                let mut offset = HEADER_SIZE + LEFT_NODE_UUID_SIZE;

                let num_keys = num_children - 1;
                for _i in 1..=num_keys {
                    let key = self.content.get_value_from_offset(offset)?;
                    offset += INDEX_INFO_SIZE;
                    res.push(key);
                }
                return Ok(res);
            }
            _ => return Err(Error::UnexpectedError),
        };
    }

    // 获得所有的孩子节点
    pub fn get_children(&self) -> Result<Vec<usize>, Error> {
        match self.node_type {
            NodeType::Unknown => return Err(Error::UnexpectedError),
            _=>{
                let num_children = self
                    .content
                    .get_value_from_offset(INDEX_NUMBER_OFFSET)?;
                let mut result = Vec::<usize>::new();
                let mut offset = HEADER_SIZE;
                for _i in 1..=num_children {
                    let child_offset = self.content.get_value_from_offset(offset)?;
                    result.push(child_offset as usize);
                    offset += INDEX_INFO_SIZE;
                }
                return Ok(result);
            }
        };
    }

}


