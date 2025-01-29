Properties that must be upheld : 
The tree must remain roughly tree shaped, to preserve log(n) operations.
there should be only 1 path that leads to a node

### Ideas
1. vary the size of nodes by tree depth.
2. presume the values of nodes are uniformly distributed between samples.
3. instead of storing pointers, calculate positions from eytzinger layout.
4. parent is prefix of child
5. array with holes in it 


### Tree Arrays

for a binary tree over the sequence

1  2  3  4  5  6  7  8  9  10 11 12 13 14 15

we'd get

                     8
         4                       12
   2           6           10          14
1     3     5     7      9    11    13    15  


So in DFS order we get an eytzinger layout

        8  4  12 2  6  10 14 1  3  5  7  9  11 13 15

index:  0  1  2  3  4  5  6  7  8  9  10 11 12 13 14

for a binary tree , each nodes children are stored at 2i + 1 and 2i + 2 

how does this insertion process work? 

#### Varying the sizes

2 ^ ( 4 + 4 + 4 + 4 + 3 + 3 + 3 + 2 + 2 + 1 ....)
                 65k              268mb           
                 
### Iterating over the tree array
coming from the perspective of : "we have a sorted array, lets tree-ify it".

we need the len, and i think thats it. 
gonna continue this in code.

### Eytzinger Forest
Ok so eytzinger trees are only valid at sizes 2^n -1 and 2^n. 
Since theyre basically sorted arrays just reshuffled, inserting into them is O(n). 
If i could find a way to merge them itd be faster. 
Found a way to merge them - split the tree by tiers - the left ones are from the lesser tree, the right 
from the greater tree. The new element that was between them is the new root.

### Porous array
I think the simplest way of implementing this is to just double up the capacity and  
intersperse elements on even indexes, inserting at the first available spot in order.

indexes wont be preserved. its sorted. thats a given.

### int map
not an array, but kinda acts like one. 
map.insert(1,2) inserts 1 with key 1.
map.insert(20,2) inserts 2 with key 20. 

internally we have a vec of keys->idx, and a vec[idx]= val. 
keys can be stored in order for relatively quick indexing.
this could be useful for the tree's rows.