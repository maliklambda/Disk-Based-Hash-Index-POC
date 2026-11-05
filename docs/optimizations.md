# Possible Optimizations
Possible optimizations are collected here.
As is the case with the [extensions](./extension.md), these are also abstract thoughts of the author.

## Optimization oportunities arising from larger hash lengths
Providing larger amounts bits to the hash (e.g. 64, or even 128 bits) has many advantages.
One of those is that the number of collisions are reduced.
Now, this introduces a possible optimization: 
Since the number of collisions for 128bit-long hashes is expected to be so rare when talking about numbers of entries south of 100 Million,
it is quite wasteful to store an offset (4 or 8 bytes) in the leaf-level (diskentries).
To address this, the linked list could be removed entirely. 
Stopping here would produce incorrect results, as collisions (as rare as they may be) must still be accounted for.
Therefore, an extension of the current architecture is needed.
### Collision table
A separate collision table (which could be implemented as a linked list), is a feasable solution 
(as collisions are rare, and reading few memory pages would bring the entire table into memory, improving locality).
To know whether or not the offset stored in the leaf-node is a pointer into the data-heap (level 3) or into the collision-table 
can be stored in a single has_collision-bit.
Another solution to knowing what table the pointer points to, is to just read the data from the data-heap. 
In most cases (non-collided values) this was the correct move.
For collided values, the length and the value are highly unlikely to match.
Therefore, upon encountering a missmatch, it is known that the pointer is for the collision table*.
This could also be implemented using memory-sections, although, this would restrict the length of the collision-table 
(unless other, very complex measures are taken).
<br>
*This approach still needs some tweaking. It is by no means completed.
