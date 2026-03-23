set camera_eye [-21.906634 25.703905 64.490958]
set camera_target [13.53033 3.935075 4.414238]
set camera_up [0 1 0]
set camera_fov 45

set maxobjects 16000
10 * { y 1 } 10 * { z 1 }  1 * { a 0.8  sat 0.9  } r1 
set background #fff


rule r1   {
  { x 1  ry 4 } r1
  xbox
}

rule r1   {
{ x 1  ry -4  } r1
xbox
}

rule r1   {
{ x 1  rz -8  s 0.95 } r1
xbox
}

rule r1   {
{ x 1  rz 8  s 0.95   } r1
xbox
}



rule r2 maxdepth 36 {
{ ry 1  ry -13 x  1.2 b 0.99 h 12  } r2 
xbox
}

rule xbox {
  { s 1.1   color #000   } grid
  { b 0.7  color #000    }  box
}

rule xbox {
 { s 1.1   color #000     } grid
 { b 0.7  color #fff      } box
}
