#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    READ,
    WRITE,
    ERASE
}

#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub id: u32, 
    pub cmd: CommandType, 
    pub logical_addr: u32,
    pub physical_addr: Option<u32>,
    pub data: *mut u8
}

#[derive(Debug, Clone, Copy)]
pub enum RequestError {
    ConnectorError, 
    StageError,
} 