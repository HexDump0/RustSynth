set camera_eye [4.020222 9.085386 43.068462]
set camera_target [-4.567633 1.726552 -4.068323]
set camera_up [0 1 0]
set camera_fov 45

set maxdepth 30

{ ry -90 color white } R1
{rx -90 color white } R1

Rule R1 {
dbox
{ z 0.6 rx 5   }  R1
}

Rule R1 {
 dbox
{ z 0.6 rx -5 }  R1
}

Rule R1 {
dbox
{ z 0.6 rz 5 }  R1
}

Rule R1 {
 dbox
{ z 0.6 rz -5 }  R1
}

Rule R1 weight 0.01 {

} 

Rule dbox {
  { s 1.5 1.6  0.5  }  box
}

Rule dbox weight 0.5 {
   { ry 90 s 0.5 } R1
}

Rule dbox weight 0.5 {
{ rx 90 s 0.5 } R1
} 