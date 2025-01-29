use generic_array::{ArrayLength,GenericArray};

// cache line will always be full. it will be constructed 
// from a ref to a slice of >= N elements.
struct CacheLine<T : Sized, N : ArrayLength>(GenericArray<T,N>);

impl<T: Sized, N:ArrayLength> CacheLine<T,N>
{
    fn new(data : &[T])
    {
        assert!(data.len() >= N::USIZE,)
    }
}

fn make_line_of_t<T>()
{

}