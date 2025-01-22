### Radix Tree
the existing impl for this one sucks, its really inefficient.

Smart strings are 
(1 bit flag + 7 bit len) + 23 bytes txt 
OR
8 byte len + 8 byte cap + 8 byte *str

thats good but i want one that packs nicely into an array.

the first 2 bits indicate the following
if the length (next 6 bits) is 0
00 next 8 bytes are a pointer to the child node cluster
01 previous string was an inserted value, next byte is the header of the next.
10 next 8 bytes are a pointer to a sibling cluster
11 the next 8 bytes are a pointer to a string (or (String, Value) in the case of a map, or just a
    Value if the length is nonzero and the next bytes code is 01)

if the length is nonzero
11 this string is just a normal sibling to the next
01 this string is the last sibling of a subnode
10 this string is a parent (prefix) to the next 
00 this is the end of the node.

and 6 bits for "len", followed by up to 63 bytes of text.
Lets call it mini string

nodes of the tree contain a fixed length 64 byte array of mini_strings.

ideally we want to skip as many 'levels' of tree as possible in one node. 


so for a string of length N, wed need to store a minimum of 1 byte for the endcap and 2 bytes for a new ministring, but that requires N bytes to already be in the tree.

such a tree would have 1 byte header per character, and 1 terminal byte per character, so our tree's overall size would be 3N. 

all those strings individually though would be n*(n+1)/ 2.

in the worst case where we're storing a ton of strings with no shared prefixes that are very long,
each one is 9 bytes and then 24 bytes for the normal string allocation. 

