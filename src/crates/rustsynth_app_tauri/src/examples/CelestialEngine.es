set maxdepth 300
set background #050510

{ x 5 y 5 z 5 s 10 color #FFFFFF } sphere
{ x 5 y 5 z 5 s 11 color #00FFFF } sphere 

{ } RingSystem
{ rx 90 s 1.1 } RingSystem
{ rz 90 s 1.2 } RingSystem

PylonEquator
{ rx 60 s 0.8 } PylonEquator
{ rx -60 s 0.8 } PylonEquator
{ rx 90 ry 90 s 0.6 } PylonEquator

rule PylonEquator {
    12 * { ry 30 } PylonHub
}

rule RingSystem {
    72 * { ry 5 } RingSegment
}

rule RingSegment {
    { x 45 s 2.5 6 3 color #334455 } box
    { x 43 s 0.5 5 1.5 color #00FFFF } box
    { x 44 y 3 rz 45 s 0.3 4 0.3 color #8899AA } box
    { x 44 y -3 rz -45 s 0.3 4 0.3 color #8899AA } box
}

rule PylonHub {
    { x 12 s 1 } Pylon
}

rule Pylon maxdepth 40 {
    { s 3   2 3 color #222233 } box
    
    { x 2.2 s 0.95 ry 3 rx 1 color #FFAA00 } Pylon
    
    { x 1 y 1.5 s 0.6 color #FF4400 } sphere
    { x 1 y -1.5 s 0.6 color #FF4400 } sphere
}
