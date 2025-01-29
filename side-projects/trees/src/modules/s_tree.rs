enum Item
{
    Slf(u32),
    Edge(u32),
}

struct TreeIter
{
    tree:  Vec<u32>,  //must assume len is a power of 2 -1. for now.
    stack: Vec<Item>, //for iterating over the edges in the tree.
}

impl TreeIter
{
    fn new(tree: Vec<u32>) -> Self
    {
        let mut stack = vec![];
        tree.get(2).and_then(|e| Some(stack.push(Item::Edge(*e))));
        if tree.len() == 0
        {
            stack.clear();
        }
        return TreeIter { tree, stack };
    }

    ///i think the trick is gonna be to iterate over the edges.
    /// an edge is defined as (node, (is left))
    /// we pop a edge off the stack, visit that node, push its kids onto the stack, then loop
    /// if there are no kids, we return that edge's node...
    fn next(&mut self)
    {
        //let node = self.stack.peek();
        //while has left
        //go left
        //return left

        //return self

        //if has right
        //return right
    }
}

/*
I wanna see if

*/

// const fn items_per_cache_line<T : Sized>() -> usize
// {
//     return 64/size_of::<T>();
// }

// fn make_sized_array<T : Sized, const N : usize>(items : &[T]) -> [T;N]
// {
//     todo!();
// }

// fn make_sized_array_helper<T: Sized, const N : usize>(items : &[T]) -> [T;N]
// {
//     let C = items_per_cache_line::<T>;
//     return make_sized_array::<T,C>(items);
// }

// struct CacheLineArray<T : Sized, const N : usize>([T;N]);

//this seems like a problem for macros.each node is a generic array of at least 2 items T.
//the macro does the size_of and chooses an appropriate generic size for the nodes.
