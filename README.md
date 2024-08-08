# coldmaps

A tool for creating heatmaps from Team Fortress 2 demos
![Screenshot](/screenshot.png)

# Tutorial video (click on the thumbnail to play)
[![Video thumbnail](https://i3.ytimg.com/vi/p-pbByda4Io/maxresdefault.jpg)](https://www.youtube.com/watch?v=p-pbByda4Io)

# Download

Check the [releases page](https://github.com/Tails8521/coldmaps/releases) and download the latest version

# How to use

1: Create a level overview screenshot, if you don't know how to, the video tutorial linked above explains it, the program isn't picky with file formats for screenshots: png, jpg or even tga are supported  
2: Take note of the coordinates of the camera at the moment you took the screenshot (x, y and the cl_leveloverview zoom level), there are two different ways: cl_showpos 1 or the values displayed in the console when you use cl_leveloverview, note that these coordinates are different but the program can understand either of them.  
Tip: You can use setpos \<x> \<y> \<z> to position yourself accurately  
3: Drag and drop the screenshot over the program's window  
4: Drag and drop the demo(s) you want to use for the heatmap  
5: Fill the camera coordinates and zoom level, don't forget to tick the checkbox corresponding to what type of coordinates you used (cl_showpos or the console)  
6: The "Export image" button lets you export the heatmap as an image file

# How to build

(This step is only needed if you want to build from source, if you're on Windows you can simply download a pre-built exe from the [releases page](https://github.com/Tails8521/coldmaps/releases))  
Download and install [Rust](https://www.rust-lang.org/learn/get-started) then `cargo build` or `cargo build --release`
