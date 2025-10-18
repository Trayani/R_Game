implement detection of CORNERS. A cell is a corner, when it is free and  it is
possible to travel around it from a vertical to horizontal direction or horizontal to vertical .

In 3x3 grid with blocked cell at (2,2), corners are (0,0), (3,0), (0,3) , (3,3)

In grid  with width:4, height: 3 and blocked at blocked cell at (2,2) and (3,2),
corners are (0,0), (4,0), (0,3) , (4,3)

So, whether a cell is corner or not is independent of observer's position.
But the observer is interested only in corners that  ARE visible AND
lead to directions that are not further visible

s... overver
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



□□□□□□□□□□□□□□□□□□□□□c□□□□□□□
n□□□□c□□□□u□u□□□□□□□□□■□□□□□□
□■■■■□□□□□□■□□□□□□□□□□■□□□□□□
c□□□□n□□□□c□c□□□□□□□□□■□□□□□□
□□□□□□□□□□□□□□□□□□□□□□□c□□□□□
u□c□□□□□□□□□□□□□□□□□□□□□□c□u□
□■□□□□□□□□□s□□□□□□□□□□□□□□■□□
u□c□□□□□□□□□□□□□□□□□□□□□□c□u□
□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□n□c□□□□□□□□
□c□n□□□□□□□□□□□□□□□■□□□□□□□□□
□□■□□□□□□□□□□□□□□□c□u□□□□□□□□
□u□c□□□□□□c□c□□□□□□□□□□□□□□□□
□□□□□□□□□□□■□□□□□□□□□□□□□□□□□
□□□□□□□□□□u□u□□□□□□□□□□□□□□□□