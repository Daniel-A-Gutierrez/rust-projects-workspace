# P Find

This is an algorithm for quickly finding numbers in a roughly uniformly distributed set.  
The minimum and maximum are taken and the query is interpolated between those to guess an accurate index.  
It doesn't surpass binary search in speed, but it does in the number of reads it makes, so it could be viable on slow storage.

