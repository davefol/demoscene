# Differential Geometry
Differential geometry is kind of like highschool geometry but using tools from
calculus and linear algebra. It's useful for demos because it gives nice
functions to generate intresting geometries.

## SLERP between two unit vectors on unit hyper-sphere
slerp(a, b, t) = sin((1-t) theta)/sin(theta) * a + sin(t * theta)/sin(theta)
where t is between 0 and 1

As a reminder, you can find theta by taking the arccos(a·b) assuming a and b
have magnitude = 1.

This also works for rotating unit quaternions since they live on the unit-3
sphere (the sphere 1 dimension higher than the common sphere).

See also: Lie-group theory, geometry