implement detection of CORNERS. A cell is a corner, when it is free and  it is
possible to travel around it from a vertical to horizontal direction or horizontal to vertical .

In 3x3 grid with blocked cell at (2,2), corners are (0,0), (3,0), (0,3) , (3,3)

In grid  with width:4, height: 3 and blocked at blocked cell at (2,2) and (3,2),
corners are (0,0), (4,0), (0,3) , (4,3)

So, whether a cell is corner or not is independent of observer's position.
But the observer is interested only in corners that  ARE within the observer's vision AND
lead to directions that are not further visible (**behind the corner**)

s... observer
■ .. blocked
▲ ... interesting corner
n ... non-interesting corner
u ... non-visible corner (cannot be seen)


□□□□□□□□□□□□□□□□□□□□□▲□□□□□□□
n□□□□▲□□□□u□u□□□□□□□□□■□□□□□□
□■■■■□□□□□□■□□□□□□□□□□■□□□□□□
▲□□□□n□□□□▲□▲□□□□□□□□□■□□□□□□
□□□□□□□□□□□□□□□□□□□□□□□▲□□□□□
u□▲□□□□□□□□□□□□□□□□□□□□□□▲□u□
□■□□□□□□□□□s□□□□□□□□□□□□□□■□□
u□▲□□□□□□□□□□□□□□□□□□□□□□▲□u□
□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□n□▲□□□□□□□□
□▲□n□□□□□□□□□□□□□□□■□□□□□□□□□
□□■□□□□□□□□□□□□□□□▲□u□□□□□□□□
□u□▲□□□□□□▲□▲□□□□□□□□□□□□□□□□
□□□□□□□□□□□■□□□□□□□□□□□□□□□□□
□□□□□□□□□□u□u□□□□□□□□□□□□□□□□


# MESSY X
is a state, where the observer is placed on two cells in the same row. His position is no "cleanly" at one cell only.
That affects vision calculation: ray casting needs to use the cell that is more conservative for given direction and left/right border. 
Primary learning files: 
[5_messy_x.txt](5_messy_x.txt)
[6_messy_x.txt](6_messy_x.txt)
[7_messy_x2.txt](7_messy_x2.txt)