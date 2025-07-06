# Linear Algebra
A very non-rigourous review of linear algebra from demos, and by extension
3D geometry. 

Goal here is to provide enough intuition to understand whats going on 
in common demo techniques.

## Vector
A tuple (fixed length list) of numbers that represents something.
A 3D point x=1, y=2, z=3 can be thought of as a vector (1, 2, 3).

This also generalizes nicely to directions. The direction "right"
in 3D can be thought of as (1, 0, 0). 

Even nicer is that this encodes "how much right" so we can encode things
like velocities and forces as well. (2, 0, 0) might mean 2 m/s second
rightward.

## Magnitude
The magnitude is the "how much" of a vector. If we imagine a vector
as an arrow pointing in some direction. The magnitude is the length of that
arrow. Sometimes this is called the L2 norm or just the norm (the L1 norm
is just the sum of all the elements of the vector). A short way to write the norm
of some vector v is ||v||.

The magnitude of (2, 0, 0), "2 units rightward" is trivially 2.

We can use the euclidean distance formula to compute non trivial magnitudes.
||(1, 2, 3)|| = sqrt(1*1 + 2*2 + 3*3) ≈ 3.74

## Normalizing
When we divide a vector by its magnitude we call it normalizing. This is kind
of a way to just talk about the direction since if we normalize a bunch of
vectors the only thing different about them is the direction they are pointing.

## Ortogonality
Once we have a concept of direction, it is useful to think about the angle
between two directions. Geometrically, when two lines cross each other
at a 90º angle we say they are perpendicular. Saying they are orthogonal
is just a more general way of describing this. By more general we mean
that you can say things like "these lines/vectors/planes/functions/spaces are orthogonal".

Similarly instead of "parallel" you may hear the more general term "linearly dependent".

(1, 0) "pointing right" is orthogonal to (0, 1) "pointing up" and parallel to
(2, 0) "pointing more right". We can also say that (1, 0) and (2, 0) are linearlly
dependent.

## Normal
Unfortunately named, a normal isn't exactly related to the concept of normalizing
a vector. Rather it has to do with orthagonality. A normal is a vector that is
orthognal to another object, ie another vector, triangle, plane, sphere, function.

In the case of a triangle, plane, sphere, etc. it can be helpful to think of the 
normal as the arrow shooting away from the surface at 90º.

A normalized normal (normal with magnitude one) crops up commonly enough that
we give it the name "unit normal". Sometimes objects are described in terms
of their unit normal. For example we may describe a plane as a unit normal 
and a displacement.

Normals are very common in lighting scenes because they can help describe how
a light ray would bounce off a surface.

## Hadamard Product
As we will see, the "product" of two vectors is an ambiguous term.
One way to multiply two vectors is by multiplying their components to 
form a new vector. We call this the Hadamard product.

```
u = [1, 2, 3]
v = [4, 5, 6]

uv = [1*1, 2*2, 3*3] # = [1, 4, 9] 
```

Usually its obvious when you want to reach for the hadamard product.
For example, if u represents some pixel intensities and v represents a lookup
table (LUT) for scaling each pixel.

## Dot Product
We can extend the hadamard product a tiny bit and make it alot more useful
by summing all the elements of the hadamard product into a single number
(a scalar). This is called the dot product.

Given two vectors, multiply corresponding elements then sum everything up.
```
u = [1, 2, 3]
v = [4, 5, 6]
s = 1*4 + 2*5 + 3*6 # = 4 + 10 + 18 = 32
```

This is already useful as a short hand way of writing the magnitude of a vector.
||v||² = v·v or ||v|| = sqrt(v·v)

It's real power comes when we look at an alternative construction.

Given two vectors, the dot product is the cosine of the smallest angle
between the two vectors scaled by the magnitude of the two vectors.

u·v = ||u|| ||v|| cos(phi)

```
u = [1, 2, 3]
v = [4, 5, 6]
phi = 0.2257261285527342 # approx 12.9 degrees
u_mag = sqrt(1*1 + 2*2 + 3*3)
v_mag = sqrt(4*4 + 5*5 + 6*6)
s = u_mag * v_mag * cos(phi) # = 32
```

But this brings up a second question: why are these two related at all?
We can think of summing the hadamard has letting each component of the
two vectors "vote" on how aligned they are. If we take u to be (1, 0), pointing right,
and v to be (0, 1), pointing up then the hadamard is (0, 0) and the dot product is 0.
the x component of the hadamard said "we are not aligned at all" while the
y component said the same and when we aggregate them the result is complete
misalignment. 

Notice that our concept of alignment implies that two vectors at right
angles are completly unalgined while to vectors that are parallel are completely
algined (positive if they are pointing in the same direction and negative otherwise).

Now if we look at the second interpretation, ||u|| ||v|| cos(phi). 
We see that the cos(phi) term captures this same relationship. 
Remember that cos(0) = 1 while cos(90º) = 0 and cos(180º) = -1.

The magnitude captures the rest of the information, i.e how big the original
vectors are. 

This gives us a very efficient way of determining:
- if two vectors are perpendicular or parallel: dot is 0 or -1, 1
- more generally the angle between two vectors: phi = arcos(u·v) / (||u|| ||v||)
- the length of the projection or "shadow" of a vector onto another one u · (v/||v||)
    I think of this as "u dot normalized v"
- work: W = Force · displacement
- power: P = Force · velocity
- and many other useful scalars

It is very very common and useful.

## Cross Product
If the dot product is a way for the components of two vectors to vote on
how similar they are, the cross product is a way for them to vote
on how dissimilar they are. Note the cross product is another vector,
not a scalar like the dot product.

It also has two classical constructions.

Similar to th dot product
u × v = ||u|| ||v|| sin(phi) n
where n is the unit normal vector.

We can intepret this similarly to how we interpreted the dot product.
sin(0) = 0, while sin(90º) = 1 so if the two vectors
are perfectly aligned the sin of the angle between them is 0 while if they
are perfectly unaligned the sin is 1.

Once we have that measure of dissimilarity from the sin function, we use that
to scale the unit normal which is a vector of magnitude 1 that is orthogonal
to both u and v. This encodes the "dissimilar direction". We also scale
by the original magnitudes of u and v like we did in the dot product to preserve
their magnitudes.

The other construction is also similar to the dot product. We can have each
component vote but this time, since we are computing dissimilarity, its
every component **but** the component we care about that votes.

```
u = [1, 2, 3]
v = [4, 5, 6]

w = [3*5 - 2*6, 3*4 - 1*6, 1*5 - 2*4] # = [3, 6, -3]
```

e.g to compute the x component of w, the y and z componets of u and v vote.

This is a hand-wavey explanation since it does say exactly why components
some times contribute positive and sometimes negative but it is enough
to show the intuitive link between the two constructions. 

Both constructions build a vector that points in a direction orthogonal to the 
source vectors and whose magnitude is proportional to the vectors and the 
angle between them.

Fortunately, based off the previous statement, there is an easy way to visualize
this vector. Instead of simply saying the magnitude is proportional, we can
say the magnitude is the exact same as the area of the parallelogram that
the vectors describe. 

This is why the area of a triangle formed by two vectors is equal to half
the magnitude of the cross product between them. 

To help clear up the ambiguity of the direction of the cross product, we can
use the right hand rule. If u is your thumb and v is your pointer finger, then
the cross product w points in the same direction as your middle finger.

The immediate application from these two definitions is finding the normal
of two vectors. If we dont care about the resulting w = u × v being unit (magnitude 1)
then we are done. If we need it to be unit we can just normalize it. This is the equivalent
of solving for n in the classical form of the cross product:

w = ||u|| ||v|| sin(phi) n
w = u × v / || u × v ||

But like the dot product there are many applications:
- torque: t = r × F (distance and direction from pivot cross force)
- angular momentum: L = r × p (distance and direction from pivot cross mass * velocity)
- magnetic force: F = qv × B (velocity cross magnetic field)
- surface normal of triangle: (b - a) × (c - a) where a, b, c are points of the triangle


