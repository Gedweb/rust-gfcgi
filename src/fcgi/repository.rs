use fcgi::entity::{
    Header,
};
use fcgi::serialize::{Serialize};

impl Serialize for Header
{
    fn read<A: Sized>(&mut self, data: A) -> &Header
    {
        self.type_ = 0;
        
        self
    }
}