extern crate byteorder;

pub trait Serialize
{
    fn read<A: Sized>(&mut self, data: A) -> &Self;
    
//    fn emit<T>(&self) -> &str;
}