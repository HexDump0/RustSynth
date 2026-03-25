# What is EisenScript?

EisenScript is a language created by Mikael Hvidtfeldt for creating 3D models with a "rule" based approach
Just like StructureSynth RustSynth is also based on `states`
A `state` defines the coordinate system,scale,color and everything
The default `state` is just [0,0,0] (But not really) with no rotation and the color red
There is only ever one state that will exist and thats it

At its core EisenScript is just composed of
- Primitives (The actual shapes)
- Transformations
- Rules
- Preprocessor stuff

> Comments start with //

# Primitives

These are just the fundamental shapes you use to make your models
To use them just type their keyword

`box` - Box/Cube/Thing with Sharp corners
`grid` - A wireframe box
`sphere` - Sphere
`line` - A line along the x axis
`cylinder` - Cylinder
`dot` - A lil tiny dot

# Transformations

This the core idea that allows for those fancy models
Transformations change the current `state`
To make a transformation open `{}` and put your transformations inside it

Lets take this simple script for example
```
{ x 1 y 1 } box
```
The transformation `{ x 1 y 1 }` will target the `box` and place it as [1,1,0]

Iterations are also possible 
```
3 * { x 1 y 1 } box
```
This will make a line of boxes
Under the hood it will just copy the `transformation` and `target` and multiplies the transformation values
So it will just be converted into
```
{ x 1 y 1 } box
{ x 2 y 2 } box
{ x 3 y 3 } box
```

All the available transformations

## Geometrical

Translation:
- `x [float]`
- `y [float]`
- `z [float]`

Rotation:
- `rx [float]`
- `ry [float]`
- `rz [float]`

Scale:
- `s [float]`

## Color

- `h [float]` - Hue, max 360 can warp around
- `s [float]` - Saturation
- `b [float]` - Brightness, 0-1
- `a [float]` - Alpha, 0-1
- `color [hex]` - Set a arbitrary hex color, can also be `random`
- `blend [color] [strength]` - Blends current color with the specified color

# Rules
Rule are the second most important concept 

You can think of rules as functions, they support recursion too (very important)

To define a rule you do
```
rule r1 {
    {x 1 y 1} box
}
r1
```
While will create a rule called `r1` and then call it
You can also apply transformations to rules the same way you would to primitives
```
rule r1 {
    {x 10 y 1} box
    {x 0.1 y 0.1}r1
}
rule r1 {
    {x 10 y 1} box
    {x -0.1 y 0.1}r1
}
r1
```
This code has alot to unwrap 
First recursion with transformation is used
And secondly there is multiple definitions for the same rule?
When you define the same rule twice which definition is used when you call the functions is randomly decided using the seed

# Terminations

EisenScript scripts are most often infinite so a external condition for termination is required
It will terminate when the `MAX OBJECTS` is exceeded by default
You can also set these settings inside the script itself
```
set maxdepth [int]
```

# Other guides
https://structuresynth.sourceforge.net/reference.php
https://structuresynth.sourceforge.net/learn.php
https://after12am.github.io/eisenscript-docs
