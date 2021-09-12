- [Project Goals](#project-goals)
  - [Features](#features)
  - [Setup](#setup)
  - [Todo](#todo)


# Project Goals
A way to visualize rust projects. Lays the groundwork for encapsulating architecture patterns in visual components.

## Features
* Web-based developer tool 
* Drag-and-drop visual components that indicate system components
* Bi-directional code generation and code interpretation 

## Setup
1. Ensure you have cargo-make installed. 
    * `cargo install cargo-make`
2.
    * If you want to run this code in a web browser:
        * `cargo make serve`
        * go to: http://127.0.0.1:4000
    * If you want to run the code natively:
        * `cargo make run`

## Todo
|  Task  |  Reasoning  |  Status  |
| -----: |  :--------  | :-----:  |
| Learn Bevy | This will be the framework for making client-side UI | :heavy_check_mark: |
| Compile to wasm | We want the UI to show up in the browser | :heavy_check_mark: |
| Add egui-bevy | This will be used for immediate graphics (which allows us to practically ignore thought of state management being separate from the application proper) | :heavy_check_mark: |
| Add toolbox | This will indicate that bevy and egui are playing together nicely | :heavy_check_mark: |
| Add tools | For now the only tools will be the edge and the node (because we are working on creating a graph-based visual editor) | :heavy_check_mark: |
| System aware of new tools | Making the tool into an enum allows us to guarantee that all tools are added at the correct touchpoints of the application | :heavy_check_mark: |
| Give the tools functionality | While it's nice that tools show up on the screen, they should actually do something. | :heavy_check_mark: |
| Determine what mouse-click hitting | Since there are theoretically many different tools available, the screen will one day be filled with different entities -- all of which will have different methods of interaction. So, it'll be important to differentiate what is being clicked... | :heavy_check_mark: |
| (Tool, ClickedComponent) functionality |  Each unique (Tool, ClickedComponent) tuple could have different functionality, the functionality given to each should be easy to specify. | :heavy_check_mark: |
| add a visualization stage to the program | This will make sense architecturally -- separating clicks/keypresses, object significance, and the visualization of the system objects. |  *in progress* | 
| Add graph structure | This will be the data type that stores most of the useful parts of the app |    |
| Add a information panel for selected items | When a node or an edge is selected, we can add that entity's information to the panel so that it can be edited/displayed easily. This might be preferable to clicking when the graph structure becomes very complex. |   |
| Make the edge tool have a visual component | Right now, there's no clear differentiator between a node and an edge |    |
| Change the bounding box to a rectangle |  When the nodes change size, it will make a lot more sense to have bounding boxes instead of circles since the circular clickbox will potentially be huge for long rectangle nodes |    |
| Move the nodes when they are click/dragged with the 'empty' selector tool |   self explanatory |    |
| lines representing edges | the edges need to have a rudimentary visual representation |    |
| separate adding/removing to the shared graph resource | Instead of adding nodes directly when a tool is used, they should be added to a graph structure when the tool is used, and a separate system should be in charge of *drawing* the graph structure |   |