use std::error::Error;
use std::fmt::{Display,Formatter};

pub(crate) type Result<T> = std::result::Result<T,NError>;
pub const ERROR_PARSE:i32 = 1;
pub const ERROR_MESSAGE_SIZE_TOO_LARGE:i32 = 2;

#[derive(Debug)]
pub struct NError{
    pub err_code:i32,
}
impl NError{
    pub fn new(err_code:i32)->Self{
        NError{
            err_code
        }
    }
    pub fn  err_description(&self)->&'static str{
        match self.err_code {
            ERROR_PARSE => "parse err",
            _=> "unknown err",
        }
    }
}
impl Error for NError{}
impl Display for NError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error>{
        write!(f,"NError [{}{}]",self.err_code,self.err_description())
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test(){
        println!("{:?}",NError::new(ERROR_PARSE));
    }
}