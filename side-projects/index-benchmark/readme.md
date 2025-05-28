# Index Benchmark

Investigating several rust datastructures performance as indexes for an in-memory database.

See results.txt for detailed results. Recap here : 

Overall : 
rudymap is surprisingly slow to build and memory hungry. 19MB for 10k items? The control is only 240k. weird.   
It performs well on random access at least, 692us is 10x faster than the B Tree and almost 3x faster than the hashmap.  
Its also 20x faster than the sorted vec.  
Its also got the unfair advantage of working with 8 byte ints instead of strings.  

Build Speed : Rudy Map (large) > Hash Map > Btree Map > Sorted Vec >> Rudy Map (Small) ~ Radix Tree   
Random Access Speed : Rudy Map (700us) 3x > Hash Map 3x > Btree Map  2x > Sorted Vec ~ Radix Tree (12ms)  
Sequential Access Speed : Sorted Vec ~ Radix Tree (13us) > BTree (17us)  
Memory Usage : RudyMap (huge , 16 bytes) > Sorted Vec (56 Bytes) > Btree Map (58 bytes) > HashMap (small : 80 Bytes) > RudyMap (small : 191 bytes) > HashMap (large : 800 Bytes)   

Sorted Vec is 2 copies of the item + 8 bytes. Makes sense.   

I think im going to do 1 last test, parallelizing sorted vec and seeing how it goes.  

...  


about 5x faster on my 8 core mobile chip. so good.  
I think for my purposes, B Tree seems the best.   
Maybe theres some postgres query to send the order of the rows in the index by column too.   