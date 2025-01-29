struct KeyString {}

struct DTreeMap<T>
{
    keys:   Vec<KeyString>,
    values: Vec<T>,
}

/*
Tree Indexing:
             [c,     g,      l,       p]
        [a,b]   [d,f]   [h,k]   [n,o]   [x,z]  //each has 2 empty slots

what if i have half nodes at the ends?
then if it has a shared prefix we'll prefer putting it there, otherwise it goes left first.

i think theres a concept of 'similarity' here


a cmp c = (0,2)
aaaaa cmp aaaac = (5,2)

> coincidence = closer
< difference = closer
coincidence matters more than difference.

so then when searching i need to follow the past of least difference.

i think  i make each node have N children, plain and simple.
so each child cluster is located at N*i?


*/
