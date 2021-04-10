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

#[macro_export]
macro_rules! parse_error {
    ( ) => {{
        //        panic!("parse error");
        return Err(NError::new(ERROR_PARSE));
    }};
}

#[derive(Debug)]
enum ParseState{
    OpStart,
    OpS,
    OpSu,
    OpSub,
    OpSubSpace,
    OpSubArg,
    OpP,
    OpPu,
    OpPub,
    OpPubSpace,
    OpPubArg,
    OpMsg,
    OpMsgFull,
}
pub struct SubArg<'a>{
    subject:&'a str,//为了避免内存分配使用str代替 String
    sid:&'a str,
    queue:Option<&'a str>,
}
pub struct PubArg<'a>{
    subject:&'a str,
    size_buf:&'a str,//1024 字符串形式，避免后续再次转换
    size:usize,//1024 整数形式
    msg:&'a [u8],
}
pub enum ParserResult<'a>{
    NoMsg,
    Sub(SubArg<'a>),
    Pub(PubArg<'a>),
}
const BUF_SIZE:usize = 512;
pub struct Parser{
    state:ParseState,
    buf:[u8;512],//消息缓冲区 如果参数+消息不超过512直接用这个，超过了必须另分配
    arg_len:usize,//参数写入计数
    msg_buf:Option<Vec<u8>>,//消息超过512时用来存消息
    msg_total_len:usize,//消息总长度
    msg_len:usize,//消息写入计数。
}
impl Parser{
    pub fn new()->Self{
        Self{
            state:ParseState::OpStart,
            buf:[0;BUF_SIZE],
            arg_len:0,
            msg_buf: None,
            msg_total_len: 0,
            msg_len: 0,
        }
    }
    /**
    对收到的字节序列进行解析，解析完毕后得到pub或者sub消息，同时有可能没有消息或者缓冲区还有其他消息
    */
    pub fn parse(&mut self,buf:&[u8])->Result<(ParserResult,usize)>{
        let mut b;
        let mut i = 0;
        while i<buf.len() {
            b = buf[i] as char;
            match self.state {
                ParseState::OpStart => {match b {
                   'S' => self.state = ParseState::OpS,
                   'P' => self.state = ParseState::OpP,
                   _=> return Err(NError::new(ERROR_PARSE)) 
                }},
                ParseState::OpS => match b {
                    'U'=>self.state = ParseState::OpSu, 
                    _=> return Err(NError::new(ERROR_PARSE)) 
                },
                ParseState::OpSu=>match b {
                    'B'=>self.state = ParseState::OpSub, 
                    _=> return Err(NError::new(ERROR_PARSE))         
                },
                ParseState::OpSub=>match b {
                    ' '|'\t' => self.state = ParseState::OpSubSpace, 
                    _=> return Err(NError::new(ERROR_PARSE))         
                },
                ParseState::OpSubSpace=> match b {
                    ' '|'\t' => {}, 
                    _=> {
                        self.state = ParseState::OpSubArg;
                        self.arg_len = 0;
                        //CONTINUE?
                    },    
                },
                ParseState::OpSubArg=>match b {
                    '\r'=>{},
                    '\n'=>{
                        //解析sub参数内容
                        self.state = ParseState::OpStart;
                        let r =  self.process_sub()?;
                        return Ok((r,i+1));
                    },
                    _ =>{
                        //收集sub参数用于后面解析
                        self.add_arg(b as u8)?;
                    }
                },
                ParseState::OpP=>match b {
                    'U'=>self.state = ParseState::OpPu, 
                    _=> return Err(NError::new(ERROR_PARSE)) 
                },
                ParseState::OpPu=>match b {
                    'B'=>self.state = ParseState::OpPub, 
                    _=> return Err(NError::new(ERROR_PARSE)) 
                },
                ParseState::OpPub=>match b {
                    ' '|'\t' =>self.state = ParseState::OpPubSpace, 
                    _=> return Err(NError::new(ERROR_PARSE)) 
                },
                ParseState::OpPubSpace=>match b {
                    ' '|'\t' => {}, 
                    _=> {
                        self.state = ParseState::OpPubArg;
                        self.arg_len = 0;
                    }
                },
                ParseState::OpPubArg=>match b {
                    '\r' => {},
                    '\n' =>{
                        self.state = ParseState::OpMsg;
                        let size = self.get_message_size()?;
                        if size==0 || size >1*1024*1024{//长度不超过1m
                            parse_error!();
                        }
                        if size + self.arg_len>BUF_SIZE{
                            self.msg_buf = Some(Vec::with_capacity(size));        
                        }
                        self.msg_total_len = size;
                    },
                    _=>{
                        //收集pub参数用于后面解析
                        self.add_arg(b as u8)?;
                    }
                }
                ParseState::OpMsg => {
                    //涉及消息长度
                    if self.msg_len < self.msg_total_len {
                        self.add_msg(b as u8);
                    } else {
                        self.state = ParseState::OpMsgFull;
                    }
                },
                ParseState::OpMsgFull=>match b {
                    '\r' => {}
                    '\n' => {
                        self.state = ParseState::OpStart;
                        let r = self.process_msg()?;
                        return Ok((r, i + 1));
                    }
                    _ => {
                        parse_error!();
                    }
                }


                _=> {}
            }    
        }
        Err(NError::new(ERROR_PARSE))
    }
    fn  add_arg(&mut self, b: u8)->Result<()>{
        if self.arg_len >= self.buf.len(){
            return Err(NError::new(ERROR_PARSE));
        }else{
            self.buf[self.arg_len] = b;
            self.arg_len+=1;
            Ok(())
        }
    }
    fn add_msg(&mut self ,b:u8)->Result<()>{
        if let Some(msgs) = self.msg_buf.as_mut(){
            msgs.push(b);
        }else{
            self.buf[self.arg_len+self.msg_len] = b;
        }
        self.msg_len+=1;
        Ok(())
    }
    fn process_sub(&self)->Result<ParserResult>{
        let buf = &self.buf[..self.arg_len];
        let ss = unsafe{std::str::from_utf8_unchecked(buf)};
        let mut  arg_buf = ["";3];
        let mut  arg_len = 0;
        for s in  ss.split(' '){
            if s.len() ==0 {
                continue;
            }
            if arg_len>=3{
                parse_error!();
            }
            arg_buf[arg_len] = s;
            arg_len+=1;
        }
        let mut sub_arg = SubArg{
            subject: arg_buf[0],
            sid: "",
            queue: None,

        };
        match arg_len {
            2 =>{
                sub_arg.sid = arg_buf[1];
            },
            3 =>{
                sub_arg.sid = arg_buf[2];
                sub_arg.queue = Some(arg_buf[1]);
            },
            _ =>{
                parse_error!();
            }
        }
        Ok(ParserResult::Sub(sub_arg))
    }
    fn process_msg(&self) -> Result<ParserResult> {
        let msg = if self.msg_buf.is_some(){
            self.msg_buf.as_ref().unwrap().as_slice()
        }else{
            &self.buf[self.arg_len..self.arg_len + self.msg_total_len]
        };
        let mut arg_buf = [""; 2];
        let mut arg_len = 0;
        let ss = unsafe { std::str::from_utf8_unchecked(&self.buf[0..self.arg_len]) };
        for s in ss.split(' ') {
            if s.len() == 0 {
                continue;
            }
            if arg_len >= 2 {
                parse_error!()
            }
            arg_buf[arg_len] = s;
            arg_len += 1;
        }
        let pub_arg = PubArg {
            subject: arg_buf[0],
            size_buf: arg_buf[1],
            size: self.msg_total_len,
            msg,
        };
        Ok(ParserResult::Pub(pub_arg))
    }
    //从接收到的pub消息中提前解析出来消息的长度字符串
    fn get_message_size(&self) -> Result<usize> {
        //缓冲区中形如top.stevenbai.top 5
        let arg_buf = &self.buf[0..self.arg_len];
        let pos = arg_buf
            .iter()
            .rev()
            .position(|b| *b == ' ' as u8 || *b == '\t' as u8);
        if pos.is_none() {
            parse_error!();
        }
        let pos = pos.unwrap();
        let size_buf = &arg_buf[arg_buf.len() - pos..];
        let szb = unsafe { std::str::from_utf8_unchecked(size_buf) };
        szb.parse::<usize>().map_err(|_| NError::new(ERROR_PARSE))
    }
}

#[cfg(test)]
mod tests{
    #[test]
    fn test(){}
}