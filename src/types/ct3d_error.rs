use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct CT3DError {
    source: Option<Box<dyn Error + 'static>>
}

impl fmt::Display for CT3DError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",format!("{:?}",self))
    }
}

// impl Error for CT3DError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         match &self.source {
//             Some(s)=>{
//                 Some(s.as_ref())
//             },
//             None => None
//         }
//     }
// }

impl CT3DError {
    pub fn new(source:Option<Box<dyn Error + 'static>>) -> CT3DError {
        CT3DError {
            source: source
        }
    }
}

impl<E: Error + 'static> From<E> for CT3DError {
    fn from(e: E) -> Self {
        Self::new(Some(Box::new(e)))
    }
}
