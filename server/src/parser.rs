/**
## pub
```
PUB <subject> <size>\r\n
<message>\r\n
```
## sub
```
SUB <subject> <sid>\r\n
SUB <subject> <queue> <sid>\r\n
```
## MSG
```
MSG <subject> <sid> <size>\r\n
<message>\r\n
```
*/
use crate::error::*;

enum ParseState{
    OpStart,
    OpS,
    OpSu,
    OpSub,
    OpSubSpace,
    OpP,
    OpPu,
    OpPub,
    OpPubSpace,
    OpMsg,
}
pub struct SubArg<'a>{
    subject:&'a str,//为了避免内存分配使用str代替 String
    sid:&'a str,
    queue:Option<&'a str>,
}
pub struct PubArg<'a>{
    subject:&'a str,
    size_buf:&'a str,//1024 字符串形式，避免后续再次转换
    size:i64,//1024 整数形式
    msg:&'a [u8],
}
pub enum ParserResult<'a>{
    NoMsg,
    Sub(SubArg<'a>),
    Pub(PubArg<'a>),
}
pub struct Parser{
    state:ParseState,
    buf:[u8;512],//消息缓冲区 如果消息不超过512直接用这个，超过了必须另分配
    arg_len:usize,//参数长度
    msg_buf:Option<Vec<u8>>,//消息超过512时使用
}
impl Parser{
    pub fn new()->Self{
        Self{
            state:ParseState::OpStart,
            buf:[0;512],
            arg_len:0,
            msg_buf: None
        }
    }
    pub fn parse(buf:&[u8])->Result<ParserResult>{
        Err(NError::new(ERROR_PARSE))
    }
}

#[cfg(test)]
mod tests{
    #[test]
    fn test(){}
}