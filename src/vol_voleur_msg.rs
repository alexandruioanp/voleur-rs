#[derive(Debug)]
pub struct VolVoleurUpdateMsg
{
    pub payload: Option<Vec<VolVoleurSinkDetails>>
}

unsafe impl Send for VolVoleurUpdateMsg {}
unsafe impl Send for VolVoleurSinkDetails {}

#[derive(Debug)]
pub struct VolVoleurSinkDetails
{
//    pub name: Vec<u8>,
    pub name: String,
//    icon: // TODO
    pub volume: u32
}
