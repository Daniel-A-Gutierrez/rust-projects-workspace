
ok so if have a sorted array , we place things going in strips into the tree in dfs order,
so left and bottom most, then its parent, then the right, then the grandparent... downright strips.

so we need to know the height of the tree beforehand. we just choose the smallest tree thatd fit it, so 
ceiling(log2(len)) = depth. 

lets do an example with len = 15
the trees height is 4.
each chlld is 2*(parent) + 1 and the root is depth 0.
each row starts on an index 2^depth - 1 when nodes are 0 indexed. 
so wed start at 2^(height-1) + 1 = 2 * 3 + 1. 
next is 