# bevy_ray_casting
## Simple ray casting implementation using Bevy game engine

Disclaimer: I don't know how actual ray casting program works, other than 
that it calculate line-line intersections. Therefore, the way I
did is probably nowhere near optimised.

I approach this problem by using svg files, because it provides
all the coordinate for every lines. However, there are many different
types of elements in svg, so I only did the line element. I used 
a crate "bevy_svg" to render the svg file. Next, at first, 
I was thinking about how can I draw the lines to the edge of the window; 
however, getting the coordinate of the edge from the direction 
is complicated. So, I ended using sine and cosine to calculate 
the circle radius of the window length. 

And now that I have the coordinates of both the wall (from svg),
and the ray (from sine & cosine); I used the line-line intersections
formula to calculate whether there are intersection(s) or not.
And, if there are more than 1 then choose the cloest one.
Now one ray is complete! The rest I just put the code into a while
loop until count reaches number of rays.

The parallel part is something that's a struggle for me. At first, I
tried to approach this by putting all the line into a vector,
and build/render all of them parallelly. However, that didn't work
because the struct Path (which is line) doesn't implement Copy.
I tried to tinker for a while, but to no avail. So, I ended with
using rayon::ThreadPool to build lines. And in the end, there
is no noticeable improvement, as far as I can see.

There is a small problem at the end, which is the test. I cannot
create test functions because bevy *cannot* run in non-main function.
